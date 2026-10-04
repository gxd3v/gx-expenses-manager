use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, SqliteConnection, SqlitePool};
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use super::{affected, transactions};
use crate::errors::AppError;
use crate::models::{TransactionKind, TransactionRecord, Transfer, TransferInput};

const GET: &str = "SELECT id, from_account_id, to_account_id, amount, date, description, created_at, updated_at \
     FROM transfers WHERE id = ?1";

const INSERT: &str = "INSERT INTO transfers (id, from_account_id, to_account_id, amount, date, description, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)";

const UPDATE: &str = "UPDATE transfers \
     SET from_account_id = ?2, to_account_id = ?3, amount = ?4, date = ?5, description = ?6, updated_at = ?7 \
     WHERE id = ?1";

const DELETE_LEGS: &str = "DELETE FROM transactions WHERE transfer_id = ?1";

const DELETE: &str = "DELETE FROM transfers WHERE id = ?1";

#[derive(FromRow)]
struct TransferRow {
    id: Hyphenated,
    from_account_id: Hyphenated,
    to_account_id: Hyphenated,
    amount: i64,
    date: NaiveDate,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TransferRow> for Transfer {
    fn from(row: TransferRow) -> Self {
        Self {
            id: Uuid::from(row.id),
            from_account_id: Uuid::from(row.from_account_id),
            to_account_id: Uuid::from(row.to_account_id),
            amount: row.amount,
            date: row.date,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct TransfersRepository {
    pool: SqlitePool,
}

impl TransfersRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: Uuid) -> Result<Transfer, AppError> {
        let row: Option<TransferRow> = sqlx::query_as(GET)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.ok_or(AppError::NotFound)?.into())
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &TransferInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        insert(&mut tx, id, input, None, now).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &TransferInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(UPDATE)
            .bind(id.hyphenated())
            .bind(input.from_account_id.hyphenated())
            .bind(input.to_account_id.hyphenated())
            .bind(input.amount)
            .bind(input.date)
            .bind(&input.description)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        affected(result.rows_affected())?;

        sqlx::query(DELETE_LEGS)
            .bind(id.hyphenated())
            .execute(&mut *tx)
            .await?;
        insert_legs(&mut tx, id, input, None, now).await?;
        tx.commit().await?;
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

pub async fn insert(
    conn: &mut SqliteConnection,
    id: Uuid,
    input: &TransferInput,
    recurrence_id: Option<Uuid>,
    now: DateTime<Utc>,
) -> Result<Uuid, AppError> {
    sqlx::query(INSERT)
        .bind(id.hyphenated())
        .bind(input.from_account_id.hyphenated())
        .bind(input.to_account_id.hyphenated())
        .bind(input.amount)
        .bind(input.date)
        .bind(&input.description)
        .bind(now)
        .execute(&mut *conn)
        .await?;

    insert_legs(conn, id, input, recurrence_id, now).await
}

async fn insert_legs(
    conn: &mut SqliteConnection,
    id: Uuid,
    input: &TransferInput,
    recurrence_id: Option<Uuid>,
    now: DateTime<Utc>,
) -> Result<Uuid, AppError> {
    let source = Uuid::now_v7();
    let legs = [
        (source, input.from_account_id, -input.amount),
        (Uuid::now_v7(), input.to_account_id, input.amount),
    ];
    for (leg, account_id, amount) in legs {
        let record = TransactionRecord {
            id: leg,
            account_id,
            category_id: None,
            kind: TransactionKind::Transfer,
            amount,
            date: input.date,
            description: input.description.clone(),
            notes: None,
            confirmed: false,
            transfer_id: Some(id),
            recurrence_id,
            one_off: false,
        };
        transactions::insert(conn, &record, now).await?;
    }
    Ok(source)
}
