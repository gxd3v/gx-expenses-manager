use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::errors::AppError;
use crate::models::{ForgottenCandidate, Reconciliation, ReconciliationStatus};

const STATUS: &str = "SELECT \
     a.initial_balance + COALESCE(SUM(t.amount), 0) AS calculated_balance, \
     a.initial_balance + COALESCE(SUM(CASE WHEN t.confirmed = 1 THEN t.amount END), 0) AS confirmed_balance, \
     COUNT(CASE WHEN t.confirmed = 0 THEN 1 END) AS unconfirmed_count, \
     COALESCE(SUM(CASE WHEN t.confirmed = 0 THEN t.amount END), 0) AS unconfirmed_total \
     FROM accounts a LEFT JOIN transactions t ON t.account_id = a.id AND t.date <= ?2 \
     WHERE a.id = ?1 GROUP BY a.id";

const CONFIRM_UNTIL: &str = "UPDATE transactions SET confirmed = 1 \
     WHERE account_id = ?1 AND date <= ?2 AND date >= ?3 AND confirmed = 0";

const INSERT: &str = "INSERT INTO reconciliations (id, account_id, date, statement_balance, calculated_balance, created_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

const LIST: &str = "SELECT id, account_id, date, statement_balance, calculated_balance, created_at \
     FROM reconciliations WHERE account_id = ?1 ORDER BY date DESC, created_at DESC";

const FORGOTTEN: &str = "SELECT t.description, \
     CASE WHEN p.name IS NULL THEN c.name ELSE p.name || ' / ' || c.name END AS category_name, \
     CAST(AVG(t.amount) AS INTEGER) AS average_amount, \
     COUNT(DISTINCT substr(t.date, 1, 7)) AS months_seen, MAX(t.date) AS last_date \
     FROM transactions t \
     LEFT JOIN categories c ON c.id = t.category_id \
     LEFT JOIN categories p ON p.id = c.parent_id \
     WHERE t.account_id = ?1 AND t.kind <> 'transfer' AND t.recurrence_id IS NULL AND trim(t.description) <> '' \
     AND t.date >= ?2 AND t.date < ?3 \
     GROUP BY lower(trim(t.description)) \
     HAVING months_seen >= 3 AND NOT EXISTS ( \
         SELECT 1 FROM transactions x WHERE x.account_id = ?1 \
         AND lower(trim(x.description)) = lower(trim(t.description)) AND x.date >= ?3 AND x.date <= ?4) \
     ORDER BY months_seen DESC, last_date DESC";

#[derive(FromRow)]
struct StatusRow {
    calculated_balance: i64,
    confirmed_balance: i64,
    unconfirmed_count: i64,
    unconfirmed_total: i64,
}

#[derive(FromRow)]
struct ReconciliationRow {
    id: Hyphenated,
    account_id: Hyphenated,
    date: NaiveDate,
    statement_balance: i64,
    calculated_balance: i64,
    created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct ForgottenRow {
    description: String,
    category_name: Option<String>,
    average_amount: i64,
    months_seen: i64,
    last_date: NaiveDate,
}

#[derive(Clone)]
pub struct ReconciliationsRepository {
    pool: SqlitePool,
}

impl ReconciliationsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn status(
        &self,
        account_id: Uuid,
        date: NaiveDate,
    ) -> Result<ReconciliationStatus, AppError> {
        let row: Option<StatusRow> = sqlx::query_as(STATUS)
            .bind(account_id.hyphenated())
            .bind(date)
            .fetch_optional(&self.pool)
            .await?;
        let row = row.ok_or(AppError::NotFound)?;

        Ok(ReconciliationStatus {
            calculated_balance: row.calculated_balance,
            confirmed_balance: row.confirmed_balance,
            unconfirmed_count: row.unconfirmed_count,
            unconfirmed_total: row.unconfirmed_total,
        })
    }

    pub async fn confirm_period(
        &self,
        account_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(CONFIRM_UNTIL)
            .bind(account_id.hyphenated())
            .bind(to)
            .bind(from)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn insert(&self, reconciliation: &Reconciliation) -> Result<(), AppError> {
        sqlx::query(INSERT)
            .bind(reconciliation.id.hyphenated())
            .bind(reconciliation.account_id.hyphenated())
            .bind(reconciliation.date)
            .bind(reconciliation.statement_balance)
            .bind(reconciliation.calculated_balance)
            .bind(reconciliation.created_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list(&self, account_id: Uuid) -> Result<Vec<Reconciliation>, AppError> {
        let rows: Vec<ReconciliationRow> = sqlx::query_as(LIST)
            .bind(account_id.hyphenated())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| Reconciliation {
                id: Uuid::from(row.id),
                account_id: Uuid::from(row.account_id),
                date: row.date,
                statement_balance: row.statement_balance,
                calculated_balance: row.calculated_balance,
                created_at: row.created_at,
            })
            .collect())
    }

    pub async fn forgotten(
        &self,
        account_id: Uuid,
        history_from: NaiveDate,
        period_from: NaiveDate,
        period_to: NaiveDate,
    ) -> Result<Vec<ForgottenCandidate>, AppError> {
        let rows: Vec<ForgottenRow> = sqlx::query_as(FORGOTTEN)
            .bind(account_id.hyphenated())
            .bind(history_from)
            .bind(period_from)
            .bind(period_to)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| ForgottenCandidate {
                description: row.description,
                category_name: row.category_name,
                average_amount: row.average_amount,
                months_seen: row.months_seen,
                last_date: row.last_date,
            })
            .collect())
    }
}
