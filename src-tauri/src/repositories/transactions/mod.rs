mod builder;
mod models;
mod queries;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::{affected, opt_id};
use crate::errors::AppError;
use crate::models::{Transaction, TransactionFilter, TransactionPage, TransactionRecord};
use models::{TotalsRow, TransactionRow};

#[derive(Clone)]
pub struct TransactionsRepository {
    pool: SqlitePool,
}

impl TransactionsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn page(
        &self,
        filter: &TransactionFilter,
        limit: i64,
        offset: i64,
    ) -> Result<TransactionPage, AppError> {
        let mut builder = QueryBuilder::<Sqlite>::new(queries::SELECT);
        builder::push_filter(&mut builder, filter);
        builder
            .push(queries::ORDER)
            .push(" LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);
        let rows: Vec<TransactionRow> = builder.build_query_as().fetch_all(&self.pool).await?;

        let mut builder = QueryBuilder::<Sqlite>::new(queries::TOTALS);
        builder::push_filter(&mut builder, filter);
        let totals: TotalsRow = builder.build_query_as().fetch_one(&self.pool).await?;

        Ok(TransactionPage {
            items: rows
                .into_iter()
                .map(Transaction::try_from)
                .collect::<Result<_, _>>()?,
            total_count: totals.total_count,
            income: totals.income,
            outcome: totals.outcome,
        })
    }

    pub async fn future(&self, today: NaiveDate) -> Result<Vec<Transaction>, AppError> {
        let rows: Vec<TransactionRow> = sqlx::query_as(queries::LIST_FUTURE)
            .bind(today)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Transaction::try_from).collect()
    }

    pub async fn get(&self, id: Uuid) -> Result<Transaction, AppError> {
        let row: Option<TransactionRow> = sqlx::query_as(queries::GET)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        row.ok_or(AppError::NotFound)?.try_into()
    }

    pub async fn insert(
        &self,
        record: &TransactionRecord,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut conn = self.pool.acquire().await?;
        insert(&mut conn, record, now).await
    }

    pub async fn update(
        &self,
        record: &TransactionRecord,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(queries::UPDATE)
            .bind(record.id.hyphenated())
            .bind(record.account_id.hyphenated())
            .bind(opt_id(record.category_id))
            .bind(record.kind.as_str())
            .bind(record.amount)
            .bind(record.date)
            .bind(&record.description)
            .bind(&record.notes)
            .bind(record.confirmed)
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

    pub async fn set_confirmed(&self, ids: &[Uuid], confirmed: bool) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let mut builder = QueryBuilder::<Sqlite>::new(queries::SET_CONFIRMED);
        builder.push_bind(confirmed).push(" WHERE id IN (");
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id.hyphenated().to_string());
        }
        builder.push(")");

        Ok(builder.build().execute(&self.pool).await?.rows_affected())
    }
}

pub async fn insert(
    conn: &mut SqliteConnection,
    record: &TransactionRecord,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query(queries::INSERT)
        .bind(record.id.hyphenated())
        .bind(record.account_id.hyphenated())
        .bind(opt_id(record.category_id))
        .bind(record.kind.as_str())
        .bind(record.amount)
        .bind(record.date)
        .bind(&record.description)
        .bind(&record.notes)
        .bind(record.confirmed)
        .bind(opt_id(record.transfer_id))
        .bind(opt_id(record.recurrence_id))
        .bind(now)
        .execute(conn)
        .await?;

    Ok(())
}
