use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use super::{affected, opt_id, to_id};
use crate::errors::AppError;
use crate::models::{Template, TemplateInput};

macro_rules! select {
    () => {
        "SELECT tp.id, tp.name, tp.account_id, a.name AS account_name, tp.category_id, \
         CASE WHEN p.name IS NULL THEN c.name ELSE p.name || ' / ' || c.name END AS category_name, \
         tp.kind, tp.amount, tp.description, tp.created_at, tp.updated_at \
         FROM templates tp \
         JOIN accounts a ON a.id = tp.account_id \
         LEFT JOIN categories c ON c.id = tp.category_id \
         LEFT JOIN categories p ON p.id = c.parent_id "
    };
}

const LIST: &str = concat!(select!(), "ORDER BY tp.name COLLATE NOCASE");

const GET: &str = concat!(select!(), "WHERE tp.id = ?1");

const INSERT: &str = "INSERT INTO templates (id, name, account_id, category_id, kind, amount, description, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)";

const UPDATE: &str = "UPDATE templates \
     SET name = ?2, account_id = ?3, category_id = ?4, kind = ?5, amount = ?6, description = ?7, updated_at = ?8 \
     WHERE id = ?1";

const DELETE: &str = "DELETE FROM templates WHERE id = ?1";

#[derive(FromRow)]
struct TemplateRow {
    id: Hyphenated,
    name: String,
    account_id: Hyphenated,
    account_name: String,
    category_id: Option<Hyphenated>,
    category_name: Option<String>,
    kind: String,
    amount: Option<i64>,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TemplateRow> for Template {
    type Error = AppError;

    fn try_from(row: TemplateRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from(row.id),
            name: row.name,
            account_id: Uuid::from(row.account_id),
            account_name: row.account_name,
            category_id: to_id(row.category_id),
            category_name: row.category_name,
            kind: row.kind.parse()?,
            amount: row.amount,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct TemplatesRepository {
    pool: SqlitePool,
}

impl TemplatesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<Template>, AppError> {
        let rows: Vec<TemplateRow> = sqlx::query_as(LIST).fetch_all(&self.pool).await?;
        rows.into_iter().map(Template::try_from).collect()
    }

    pub async fn get(&self, id: Uuid) -> Result<Template, AppError> {
        let row: Option<TemplateRow> = sqlx::query_as(GET)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;
        row.ok_or(AppError::NotFound)?.try_into()
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &TemplateInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(INSERT)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.account_id.hyphenated())
            .bind(opt_id(input.category_id))
            .bind(input.kind.as_str())
            .bind(input.amount)
            .bind(&input.description)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &TemplateInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(UPDATE)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.account_id.hyphenated())
            .bind(opt_id(input.category_id))
            .bind(input.kind.as_str())
            .bind(input.amount)
            .bind(&input.description)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(DELETE)
            .bind(id.hyphenated())
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }
}
