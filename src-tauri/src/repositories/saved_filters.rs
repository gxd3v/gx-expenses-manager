use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use super::affected;
use crate::errors::AppError;

const LIST: &str =
    "SELECT id, name, filter, created_at FROM saved_filters ORDER BY name COLLATE NOCASE";

const INSERT: &str =
    "INSERT INTO saved_filters (id, name, filter, created_at) VALUES (?1, ?2, ?3, ?4)";

const DELETE: &str = "DELETE FROM saved_filters WHERE id = ?1";

#[derive(FromRow)]
struct SavedFilterRow {
    id: Hyphenated,
    name: String,
    filter: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SavedFilter {
    pub id: Uuid,
    pub name: String,
    pub filter: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SavedFiltersRepository {
    pool: SqlitePool,
}

impl SavedFiltersRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<SavedFilter>, AppError> {
        let rows: Vec<SavedFilterRow> = sqlx::query_as(LIST).fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| SavedFilter {
                id: Uuid::from(row.id),
                name: row.name,
                filter: row.filter,
                created_at: row.created_at,
            })
            .collect())
    }

    pub async fn insert(&self, filter: &SavedFilter) -> Result<(), AppError> {
        sqlx::query(INSERT)
            .bind(filter.id.hyphenated())
            .bind(&filter.name)
            .bind(&filter.filter)
            .bind(filter.created_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(DELETE)
            .bind(id.hyphenated())
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }
}
