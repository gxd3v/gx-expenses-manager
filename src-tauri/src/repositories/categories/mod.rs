mod models;
mod queries;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::{affected, opt_id};
use crate::errors::AppError;
use crate::models::{Category, CategoryInput};
use models::CategoryRow;

#[derive(Clone)]
pub struct CategoriesRepository {
    pool: SqlitePool,
}

impl CategoriesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, include_archived: bool) -> Result<Vec<Category>, AppError> {
        let rows: Vec<CategoryRow> = sqlx::query_as(queries::LIST)
            .bind(include_archived)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(Category::try_from).collect()
    }

    pub async fn get(&self, id: Uuid) -> Result<Category, AppError> {
        let row: Option<CategoryRow> = sqlx::query_as(queries::GET)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        row.ok_or(AppError::NotFound)?.try_into()
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &CategoryInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(queries::INSERT)
            .bind(id.hyphenated())
            .bind(opt_id(input.parent_id))
            .bind(&input.name)
            .bind(input.kind.as_str())
            .bind(&input.icon)
            .bind(&input.color)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &CategoryInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::UPDATE)
            .bind(id.hyphenated())
            .bind(opt_id(input.parent_id))
            .bind(&input.name)
            .bind(input.kind.as_str())
            .bind(&input.icon)
            .bind(&input.color)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn set_archived(
        &self,
        id: Uuid,
        archived_at: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::SET_ARCHIVED)
            .bind(id.hyphenated())
            .bind(archived_at)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn has_children(&self, id: Uuid) -> Result<bool, AppError> {
        Ok(sqlx::query_scalar(queries::HAS_CHILDREN)
            .bind(id.hyphenated())
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn in_use(&self, id: Uuid) -> Result<bool, AppError> {
        Ok(sqlx::query_scalar(queries::IN_USE)
            .bind(id.hyphenated())
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn delete(&self, id: Uuid, reassign_to: Option<Uuid>) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        if let Some(target) = reassign_to {
            for query in queries::REASSIGN {
                sqlx::query(query)
                    .bind(id.hyphenated())
                    .bind(target.hyphenated())
                    .execute(&mut *tx)
                    .await?;
            }
        }

        let result = sqlx::query(queries::DELETE)
            .bind(id.hyphenated())
            .execute(&mut *tx)
            .await?;
        affected(result.rows_affected())?;
        tx.commit().await?;
        Ok(())
    }
}
