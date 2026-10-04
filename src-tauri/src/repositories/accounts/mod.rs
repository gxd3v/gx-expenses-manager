mod models;
mod queries;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::affected;
use crate::errors::AppError;
use crate::models::{Account, AccountInput};
use models::AccountRow;

#[derive(Clone)]
pub struct AccountsRepository {
    pool: SqlitePool,
}

impl AccountsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        today: NaiveDate,
        include_archived: bool,
    ) -> Result<Vec<Account>, AppError> {
        let rows: Vec<AccountRow> = sqlx::query_as(queries::LIST)
            .bind(today)
            .bind(include_archived)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(Account::try_from).collect()
    }

    pub async fn get(&self, today: NaiveDate, id: Uuid) -> Result<Account, AppError> {
        let row: Option<AccountRow> = sqlx::query_as(queries::GET)
            .bind(today)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        row.ok_or(AppError::NotFound)?.try_into()
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &AccountInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(queries::INSERT)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.kind.as_str())
            .bind(&input.currency)
            .bind(input.initial_balance)
            .bind(&input.color)
            .bind(&input.icon)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &AccountInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::UPDATE)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.kind.as_str())
            .bind(&input.currency)
            .bind(input.initial_balance)
            .bind(&input.color)
            .bind(&input.icon)
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

    pub async fn in_use(&self, id: Uuid) -> Result<bool, AppError> {
        Ok(sqlx::query_scalar(queries::IN_USE)
            .bind(id.hyphenated())
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(queries::DELETE)
            .bind(id.hyphenated())
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }
}
