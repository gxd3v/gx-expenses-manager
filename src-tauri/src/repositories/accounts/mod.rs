mod models;
mod queries;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

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

    pub async fn list(&self, include_archived: bool) -> Result<Vec<Account>, AppError> {
        let rows: Vec<AccountRow> = sqlx::query_as(queries::LIST)
            .bind(include_archived)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(Account::try_from).collect()
    }

    pub async fn get(&self, id: Uuid) -> Result<Account, AppError> {
        let row: Option<AccountRow> = sqlx::query_as(queries::GET)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        row.ok_or(AppError::NotFound)?.try_into()
    }

    pub async fn insert(&self, account: &Account) -> Result<(), AppError> {
        sqlx::query(queries::INSERT)
            .bind(account.id.hyphenated())
            .bind(&account.name)
            .bind(account.kind.as_str())
            .bind(&account.currency)
            .bind(account.initial_balance)
            .bind(&account.color)
            .bind(account.created_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update(&self, id: Uuid, input: &AccountInput, now: DateTime<Utc>) -> Result<(), AppError> {
        let result = sqlx::query(queries::UPDATE)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.kind.as_str())
            .bind(&input.currency)
            .bind(input.initial_balance)
            .bind(&input.color)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn archive(&self, id: Uuid, now: DateTime<Utc>) -> Result<(), AppError> {
        let result = sqlx::query(queries::ARCHIVE)
            .bind(id.hyphenated())
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }
}

fn affected(rows: u64) -> Result<(), AppError> {
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
