use std::path::Path;

use serde_json::{Map, Value, json};
use sqlx::query::Query;
use sqlx::sqlite::{SqliteArguments, SqliteRow};
use sqlx::{Column, Row, Sqlite, SqliteConnection, SqlitePool, TypeInfo, ValueRef};

use crate::errors::AppError;

pub const TABLES: [&str; 13] = [
    "accounts",
    "categories",
    "transfers",
    "credits",
    "recurrences",
    "transactions",
    "recurrence_occurrences",
    "credit_payments",
    "goals",
    "templates",
    "reconciliations",
    "saved_filters",
    "settings",
];

const SCHEMA_VERSION: &str = "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations";

#[derive(Debug, Clone, Default)]
pub struct LoadResult {
    pub inserted: u64,
    pub skipped: u64,
}

#[derive(Clone)]
pub struct BackupsRepository {
    pool: SqlitePool,
}

impl BackupsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn schema_version(&self) -> Result<i64, AppError> {
        Ok(sqlx::query_scalar(SCHEMA_VERSION)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn snapshot(&self, path: &Path) -> Result<(), AppError> {
        sqlx::query("VACUUM INTO ?1")
            .bind(path.to_string_lossy())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn dump(&self) -> Result<Map<String, Value>, AppError> {
        let mut tables = Map::new();
        for table in TABLES {
            let rows = sqlx::query(&format!("SELECT * FROM {table} ORDER BY rowid"))
                .fetch_all(&self.pool)
                .await?;
            let rows = rows
                .iter()
                .map(row_to_json)
                .collect::<Result<Vec<_>, _>>()?;
            tables.insert(table.into(), Value::Array(rows));
        }
        Ok(tables)
    }

    pub async fn conflicts(&self, tables: &Map<String, Value>) -> Result<u64, AppError> {
        let mut conn = self.pool.acquire().await?;
        let mut total = 0;
        for table in TABLES {
            let keys = primary_key(&mut conn, table).await?;
            for row in rows(tables, table)? {
                total += u64::from(exists(&mut conn, table, &keys, row).await?);
            }
        }
        Ok(total)
    }

    pub async fn load(
        &self,
        tables: &Map<String, Value>,
        replace: bool,
    ) -> Result<LoadResult, AppError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("PRAGMA defer_foreign_keys = ON")
            .execute(&mut *tx)
            .await?;

        if replace {
            for table in TABLES.iter().rev() {
                sqlx::query(&format!("DELETE FROM {table}"))
                    .execute(&mut *tx)
                    .await?;
            }
        }

        let mut result = LoadResult::default();
        for table in TABLES {
            let columns = columns(&mut tx, table).await?;
            for row in rows(tables, table)? {
                let inserted = insert_row(&mut tx, table, &columns, row).await?;
                result.inserted += inserted;
                result.skipped += 1 - inserted;
            }
        }

        tx.commit().await.map_err(|_| {
            AppError::InvalidBackup("os dados violam as relações entre tabelas".into())
        })?;
        Ok(result)
    }
}

pub fn rows<'a>(tables: &'a Map<String, Value>, table: &str) -> Result<&'a [Value], AppError> {
    match tables.get(table) {
        None => Ok(&[]),
        Some(Value::Array(rows)) => Ok(rows),
        Some(_) => Err(AppError::InvalidBackup(format!(
            "a tabela '{table}' não é uma lista"
        ))),
    }
}

fn row_to_json(row: &SqliteRow) -> Result<Value, AppError> {
    let mut object = Map::new();
    for column in row.columns() {
        object.insert(column.name().into(), cell(row, column.ordinal())?);
    }
    Ok(Value::Object(object))
}

fn cell(row: &SqliteRow, index: usize) -> Result<Value, AppError> {
    let kind = {
        let raw = row.try_get_raw(index)?;
        if raw.is_null() {
            return Ok(Value::Null);
        }
        raw.type_info().name().to_string()
    };

    Ok(match kind.as_str() {
        "INTEGER" => json!(row.try_get::<i64, _>(index)?),
        "REAL" => json!(row.try_get::<f64, _>(index)?),
        _ => json!(row.try_get::<String, _>(index)?),
    })
}

async fn columns(
    conn: &mut SqliteConnection,
    table: &str,
) -> Result<Vec<(String, bool)>, AppError> {
    let rows = sqlx::query(&format!("PRAGMA table_info({table})"))
        .fetch_all(conn)
        .await?;
    rows.iter()
        .map(|row| {
            Ok((
                row.try_get::<String, _>("name")?,
                row.try_get::<i64, _>("pk")? > 0,
            ))
        })
        .collect()
}

async fn primary_key(conn: &mut SqliteConnection, table: &str) -> Result<Vec<String>, AppError> {
    let columns = columns(conn, table).await?;
    Ok(columns
        .into_iter()
        .filter(|(_, pk)| *pk)
        .map(|(name, _)| name)
        .collect())
}

async fn exists(
    conn: &mut SqliteConnection,
    table: &str,
    keys: &[String],
    row: &Value,
) -> Result<bool, AppError> {
    let object = as_object(table, row)?;
    let condition = keys
        .iter()
        .map(|key| format!("{key} = ?"))
        .collect::<Vec<_>>()
        .join(" AND ");
    let sql = format!("SELECT EXISTS (SELECT 1 FROM {table} WHERE {condition})");

    let mut query = sqlx::query(&sql);
    for key in keys {
        query = bind(query, object.get(key).unwrap_or(&Value::Null))?;
    }
    Ok(query.fetch_one(conn).await?.try_get::<bool, _>(0)?)
}

async fn insert_row(
    conn: &mut SqliteConnection,
    table: &str,
    columns: &[(String, bool)],
    row: &Value,
) -> Result<u64, AppError> {
    let object = as_object(table, row)?;
    if let Some(unknown) = object
        .keys()
        .find(|key| !columns.iter().any(|(name, _)| name == *key))
    {
        return Err(AppError::InvalidBackup(format!(
            "coluna desconhecida '{table}.{unknown}'"
        )));
    }

    let names = object.keys().cloned().collect::<Vec<_>>().join(", ");
    let placeholders = vec!["?"; object.len()].join(", ");
    let sql = format!("INSERT OR IGNORE INTO {table} ({names}) VALUES ({placeholders})");

    let mut query = sqlx::query(&sql);
    for value in object.values() {
        query = bind(query, value)?;
    }
    Ok(query.execute(conn).await?.rows_affected())
}

fn as_object<'a>(table: &str, row: &'a Value) -> Result<&'a Map<String, Value>, AppError> {
    row.as_object()
        .ok_or_else(|| AppError::InvalidBackup(format!("linha inválida na tabela '{table}'")))
}

fn bind<'q>(
    query: Query<'q, Sqlite, SqliteArguments<'q>>,
    value: &Value,
) -> Result<Query<'q, Sqlite, SqliteArguments<'q>>, AppError> {
    match value {
        Value::Null => Ok(query.bind(None::<String>)),
        Value::Bool(value) => Ok(query.bind(*value)),
        Value::Number(number) if number.is_i64() => Ok(query.bind(number.as_i64())),
        Value::Number(number) => Ok(query.bind(number.as_f64())),
        Value::String(text) => Ok(query.bind(text.clone())),
        _ => Err(AppError::InvalidBackup("valor não suportado".into())),
    }
}
