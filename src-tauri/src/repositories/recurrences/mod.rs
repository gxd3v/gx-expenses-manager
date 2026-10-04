mod models;
mod queries;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::{affected, credits, opt_id, transactions};
use crate::errors::AppError;
use crate::models::{
    CreditPayment, OccurrenceOverride, Recurrence, RecurrenceInput, TransactionRecord,
};
use models::{OverrideRow, RecurrenceRow};

#[derive(Clone)]
pub struct RecurrencesRepository {
    pool: SqlitePool,
}

impl RecurrencesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<Recurrence>, AppError> {
        let rows: Vec<RecurrenceRow> = sqlx::query_as(queries::LIST).fetch_all(&self.pool).await?;
        rows.into_iter().map(Recurrence::try_from).collect()
    }

    pub async fn get(&self, id: Uuid) -> Result<Recurrence, AppError> {
        let row: Option<RecurrenceRow> = sqlx::query_as(queries::GET)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        row.ok_or(AppError::NotFound)?.try_into()
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &RecurrenceInput,
        credit_id: Option<Uuid>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(queries::INSERT)
            .bind(id.hyphenated())
            .bind(input.account_id.hyphenated())
            .bind(opt_id(input.category_id))
            .bind(input.kind.as_str())
            .bind(input.amount)
            .bind(&input.description)
            .bind(input.start_date)
            .bind(input.end_date)
            .bind(input.frequency.unit.as_str())
            .bind(input.frequency.interval)
            .bind(opt_id(credit_id))
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &RecurrenceInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::UPDATE)
            .bind(id.hyphenated())
            .bind(input.account_id.hyphenated())
            .bind(opt_id(input.category_id))
            .bind(input.kind.as_str())
            .bind(input.amount)
            .bind(&input.description)
            .bind(input.start_date)
            .bind(input.end_date)
            .bind(input.frequency.unit.as_str())
            .bind(input.frequency.interval)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn set_paused(
        &self,
        id: Uuid,
        paused_at: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::SET_PAUSED)
            .bind(id.hyphenated())
            .bind(paused_at)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn set_end(
        &self,
        id: Uuid,
        end_date: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::SET_END)
            .bind(id.hyphenated())
            .bind(end_date)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(queries::DELETE)
            .bind(id.hyphenated())
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }

    pub async fn overrides(&self) -> Result<Vec<OccurrenceOverride>, AppError> {
        let rows: Vec<OverrideRow> = sqlx::query_as(queries::OVERRIDES)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(OccurrenceOverride::try_from).collect()
    }

    pub async fn upsert_override(&self, change: &OccurrenceOverride) -> Result<(), AppError> {
        let mut conn = self.pool.acquire().await?;
        upsert_override(&mut conn, change).await
    }

    pub async fn delete_override(
        &self,
        recurrence_id: Uuid,
        occurrence_date: NaiveDate,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::DELETE_OVERRIDE)
            .bind(recurrence_id.hyphenated())
            .bind(occurrence_date)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn materialize(
        &self,
        record: &TransactionRecord,
        change: &OccurrenceOverride,
        payment: Option<&CreditPayment>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        transactions::insert(&mut tx, record, now).await?;
        upsert_override(&mut tx, change).await?;
        if let Some(payment) = payment {
            credits::insert_payment(&mut tx, payment).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

async fn upsert_override(
    conn: &mut SqliteConnection,
    change: &OccurrenceOverride,
) -> Result<(), AppError> {
    sqlx::query(queries::UPSERT_OVERRIDE)
        .bind(change.recurrence_id.hyphenated())
        .bind(change.occurrence_date)
        .bind(change.status.as_str())
        .bind(change.amount)
        .bind(change.date)
        .bind(opt_id(change.transaction_id))
        .execute(conn)
        .await?;

    Ok(())
}
