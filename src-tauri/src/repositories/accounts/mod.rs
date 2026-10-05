mod models;
mod queries;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::affected;
use crate::errors::AppError;
use crate::models::{Account, AccountInput, Interest, InterestTier};
use models::{AccountRow, TierRow};

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

        let accounts = rows
            .into_iter()
            .map(Account::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        self.with_interest(accounts).await
    }

    pub async fn get(&self, today: NaiveDate, id: Uuid) -> Result<Account, AppError> {
        let row: Option<AccountRow> = sqlx::query_as(queries::GET)
            .bind(today)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        let account = row.ok_or(AppError::NotFound)?.try_into()?;
        let mut accounts = self.with_interest(vec![account]).await?;
        Ok(accounts.remove(0))
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &AccountInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(queries::INSERT)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.kind.as_str())
            .bind(&input.currency)
            .bind(input.initial_balance)
            .bind(&input.color)
            .bind(&input.icon)
            .bind(input.interest.as_ref().map(|i| i.period_months))
            .bind(input.overdraft_limit)
            .bind(now)
            .execute(&mut *tx)
            .await?;

        save_tiers(&mut tx, id, input.interest.as_ref()).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &AccountInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(queries::UPDATE)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.kind.as_str())
            .bind(&input.currency)
            .bind(input.initial_balance)
            .bind(&input.color)
            .bind(&input.icon)
            .bind(input.interest.as_ref().map(|i| i.period_months))
            .bind(input.overdraft_limit)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        affected(result.rows_affected())?;

        save_tiers(&mut tx, id, input.interest.as_ref()).await?;
        tx.commit().await?;
        Ok(())
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

    async fn with_interest(&self, mut accounts: Vec<Account>) -> Result<Vec<Account>, AppError> {
        let tiers: Vec<TierRow> = sqlx::query_as(queries::TIERS).fetch_all(&self.pool).await?;
        for account in &mut accounts {
            if let Some(interest) = account.interest.as_mut() {
                interest.tiers = tiers
                    .iter()
                    .filter(|t| Uuid::from(t.account_id) == account.id)
                    .map(|t| InterestTier {
                        min_balance: t.min_balance,
                        rate: t.rate,
                    })
                    .collect();
            }
        }
        Ok(accounts)
    }
}

async fn save_tiers(
    conn: &mut SqliteConnection,
    id: Uuid,
    interest: Option<&Interest>,
) -> Result<(), AppError> {
    sqlx::query(queries::DELETE_TIERS)
        .bind(id.hyphenated())
        .execute(&mut *conn)
        .await?;
    for tier in interest.map_or(&[][..], |i| &i.tiers[..]) {
        sqlx::query(queries::INSERT_TIER)
            .bind(id.hyphenated())
            .bind(tier.min_balance)
            .bind(tier.rate)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}
