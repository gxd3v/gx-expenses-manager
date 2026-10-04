mod models;
mod queries;

use chrono::{DateTime, Utc};
use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::{affected, opt_id, transactions};
use crate::errors::AppError;
use crate::models::{Credit, CreditInput, CreditPayment, TransactionRecord};
use models::{CreditRow, PaymentRow};

#[derive(Clone)]
pub struct CreditsRepository {
    pool: SqlitePool,
}

impl CreditsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, include_archived: bool) -> Result<Vec<Credit>, AppError> {
        let rows: Vec<CreditRow> = sqlx::query_as(queries::LIST)
            .bind(include_archived)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Credit::try_from).collect()
    }

    pub async fn get(&self, id: Uuid) -> Result<Credit, AppError> {
        let row: Option<CreditRow> = sqlx::query_as(queries::GET)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        row.ok_or(AppError::NotFound)?.try_into()
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &CreditInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(queries::INSERT)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(&input.institution)
            .bind(input.principal)
            .bind(input.opening_balance)
            .bind(input.annual_rate)
            .bind(input.installment)
            .bind(input.frequency.unit.as_str())
            .bind(input.frequency.interval)
            .bind(input.start_date)
            .bind(input.end_date)
            .bind(input.installments)
            .bind(opt_id(input.account_id))
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &CreditInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::UPDATE)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(&input.institution)
            .bind(input.principal)
            .bind(input.opening_balance)
            .bind(input.annual_rate)
            .bind(input.installment)
            .bind(input.frequency.unit.as_str())
            .bind(input.frequency.interval)
            .bind(input.start_date)
            .bind(input.end_date)
            .bind(input.installments)
            .bind(opt_id(input.account_id))
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

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(queries::DELETE)
            .bind(id.hyphenated())
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }

    pub async fn payments(&self, credit_id: Option<Uuid>) -> Result<Vec<CreditPayment>, AppError> {
        let rows: Vec<PaymentRow> = sqlx::query_as(queries::PAYMENTS)
            .bind(opt_id(credit_id))
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(CreditPayment::from).collect())
    }

    pub async fn insert_payment(
        &self,
        payment: &CreditPayment,
        transaction: Option<&TransactionRecord>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        if let Some(record) = transaction {
            transactions::insert(&mut tx, record, payment.created_at).await?;
        }
        insert_payment(&mut tx, payment).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_payment(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(queries::DELETE_PAYMENT)
            .bind(id.hyphenated())
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }
}

pub async fn insert_payment(
    conn: &mut SqliteConnection,
    payment: &CreditPayment,
) -> Result<(), AppError> {
    sqlx::query(queries::INSERT_PAYMENT)
        .bind(payment.id.hyphenated())
        .bind(payment.credit_id.hyphenated())
        .bind(opt_id(payment.transaction_id))
        .bind(payment.date)
        .bind(payment.principal)
        .bind(payment.interest)
        .bind(payment.created_at)
        .execute(conn)
        .await?;

    Ok(())
}
