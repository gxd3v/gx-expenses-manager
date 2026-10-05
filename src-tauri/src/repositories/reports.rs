use chrono::NaiveDate;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use super::{opt_id, to_id};
use crate::errors::AppError;
use crate::models::{CategoryAmount, CategoryGrouping, EntryKind, MonthlyTotal};

const MONTHLY_TOTALS: &str = "SELECT substr(t.date, 1, 7) AS month, \
     COALESCE(SUM(CASE WHEN t.kind = 'income' THEN t.amount END), 0) AS income, \
     COALESCE(SUM(CASE WHEN t.kind = 'outcome' THEN -t.amount END), 0) AS outcome \
     FROM transactions t \
     WHERE t.kind <> 'transfer' AND t.date >= ?1 AND t.date <= ?2 AND (?3 IS NULL OR t.account_id = ?3) \
     AND (?4 = 0 OR t.one_off = 0) \
     GROUP BY month ORDER BY month";

const BY_PARENT: &str = "SELECT COALESCE(p.id, c.id) AS category_id, COALESCE(p.name, c.name, 'Sem categoria') AS name, \
     COALESCE(p.color, c.color) AS color, SUM(abs(t.amount)) AS amount \
     FROM transactions t \
     LEFT JOIN categories c ON c.id = t.category_id \
     LEFT JOIN categories p ON p.id = c.parent_id \
     WHERE t.kind = ?1 AND t.date >= ?2 AND t.date <= ?3 AND (?4 IS NULL OR t.account_id = ?4) \
     GROUP BY 1 ORDER BY amount DESC";

const BY_LEAF: &str = "SELECT c.id AS category_id, \
     COALESCE(CASE WHEN p.name IS NULL THEN c.name ELSE p.name || ' / ' || c.name END, 'Sem categoria') AS name, \
     COALESCE(c.color, p.color) AS color, SUM(abs(t.amount)) AS amount \
     FROM transactions t \
     LEFT JOIN categories c ON c.id = t.category_id \
     LEFT JOIN categories p ON p.id = c.parent_id \
     WHERE t.kind = ?1 AND t.date >= ?2 AND t.date <= ?3 AND (?4 IS NULL OR t.account_id = ?4) \
     GROUP BY 1 ORDER BY amount DESC";

const CATEGORY_MONTHS: &str = "SELECT COALESCE(p.id, c.id) AS category_id, COALESCE(p.name, c.name, 'Sem categoria') AS name, \
     COALESCE(p.color, c.color) AS color, substr(t.date, 1, 7) AS month, SUM(-t.amount) AS amount, \
     SUM(CASE WHEN t.one_off = 0 THEN -t.amount ELSE 0 END) AS regular \
     FROM transactions t \
     LEFT JOIN categories c ON c.id = t.category_id \
     LEFT JOIN categories p ON p.id = c.parent_id \
     WHERE t.kind = 'outcome' AND t.date >= ?1 AND t.date <= ?2 \
     GROUP BY 1, month";

const ACCOUNT_MONTHS: &str = "SELECT account_id, substr(date, 1, 7) AS month, SUM(amount) AS amount \
     FROM transactions WHERE date <= ?1 GROUP BY account_id, month ORDER BY month";

const DAILY_CHANGES: &str = "SELECT t.date, SUM(t.amount) AS amount FROM transactions t \
     JOIN accounts a ON a.id = t.account_id \
     WHERE t.date <= ?1 AND (?2 IS NULL OR t.account_id = ?2) \
     AND (?2 IS NOT NULL OR (a.archived_at IS NULL AND a.kind NOT IN ('credit_card', 'meal'))) \
     GROUP BY t.date ORDER BY t.date";

const VARIABLE_AVERAGES: &str = "SELECT t.account_id, \
     COALESCE(SUM(CASE WHEN t.kind = 'income' AND t.date < ?2 THEN t.amount END), 0) AS income, \
     COALESCE(SUM(CASE WHEN t.kind = 'outcome' AND t.date < ?2 THEN -t.amount END), 0) AS outcome, \
     COALESCE(SUM(CASE WHEN t.kind = 'income' AND t.date >= ?2 THEN t.amount END), 0) AS current_income, \
     COALESCE(SUM(CASE WHEN t.kind = 'outcome' AND t.date >= ?2 THEN -t.amount END), 0) AS current_outcome \
     FROM transactions t \
     WHERE t.kind <> 'transfer' AND t.recurrence_id IS NULL AND t.one_off = 0 \
     AND NOT EXISTS (SELECT 1 FROM credit_payments cp WHERE cp.transaction_id = t.id) \
     AND NOT EXISTS (SELECT 1 FROM recurrences r \
         WHERE r.paused_at IS NULL AND (r.end_date IS NULL OR r.end_date >= ?3) \
         AND r.account_id = t.account_id AND r.kind = t.kind \
         AND (lower(trim(r.description)) = lower(trim(t.description)) OR r.category_id = t.category_id)) \
     AND t.date >= ?1 AND t.date <= ?3 \
     GROUP BY t.account_id";

#[derive(FromRow)]
struct MonthlyRow {
    month: String,
    income: i64,
    outcome: i64,
}

#[derive(FromRow)]
struct CategoryRow {
    category_id: Option<Hyphenated>,
    name: String,
    color: Option<String>,
    amount: i64,
}

#[derive(FromRow)]
pub struct CategoryMonthRow {
    pub category_id: Option<Hyphenated>,
    pub name: String,
    pub color: Option<String>,
    pub month: String,
    pub amount: i64,
    pub regular: i64,
}

#[derive(FromRow)]
pub struct AccountMonthRow {
    pub account_id: Hyphenated,
    pub month: String,
    pub amount: i64,
}

#[derive(FromRow)]
struct DailyRow {
    date: NaiveDate,
    amount: i64,
}

#[derive(FromRow)]
pub struct VariableRow {
    pub account_id: Hyphenated,
    pub income: i64,
    pub outcome: i64,
    pub current_income: i64,
    pub current_outcome: i64,
}

#[derive(Clone)]
pub struct ReportsRepository {
    pool: SqlitePool,
}

impl ReportsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn monthly_totals(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        account_id: Option<Uuid>,
        regular_only: bool,
    ) -> Result<Vec<MonthlyTotal>, AppError> {
        let rows: Vec<MonthlyRow> = sqlx::query_as(MONTHLY_TOTALS)
            .bind(from)
            .bind(to)
            .bind(opt_id(account_id))
            .bind(regular_only)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(MonthlyTotal {
                    month: parse_month(&row.month)?,
                    income: row.income,
                    outcome: row.outcome,
                })
            })
            .collect()
    }

    pub async fn by_category(
        &self,
        kind: EntryKind,
        from: NaiveDate,
        to: NaiveDate,
        grouping: CategoryGrouping,
        account_id: Option<Uuid>,
    ) -> Result<Vec<CategoryAmount>, AppError> {
        let query = match grouping {
            CategoryGrouping::Parent => BY_PARENT,
            CategoryGrouping::Leaf => BY_LEAF,
        };
        let rows: Vec<CategoryRow> = sqlx::query_as(query)
            .bind(kind.as_str())
            .bind(from)
            .bind(to)
            .bind(opt_id(account_id))
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| CategoryAmount {
                category_id: to_id(row.category_id),
                name: row.name,
                color: row.color,
                amount: row.amount,
            })
            .collect())
    }

    pub async fn category_months(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<CategoryMonthRow>, AppError> {
        Ok(sqlx::query_as(CATEGORY_MONTHS)
            .bind(from)
            .bind(to)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn account_months(&self, until: NaiveDate) -> Result<Vec<AccountMonthRow>, AppError> {
        Ok(sqlx::query_as(ACCOUNT_MONTHS)
            .bind(until)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn daily_changes(
        &self,
        until: NaiveDate,
        account_id: Option<Uuid>,
    ) -> Result<Vec<(NaiveDate, i64)>, AppError> {
        let rows: Vec<DailyRow> = sqlx::query_as(DAILY_CHANGES)
            .bind(until)
            .bind(opt_id(account_id))
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|row| (row.date, row.amount)).collect())
    }

    pub async fn variable_totals(
        &self,
        from: NaiveDate,
        current_month: NaiveDate,
        today: NaiveDate,
    ) -> Result<Vec<VariableRow>, AppError> {
        Ok(sqlx::query_as(VARIABLE_AVERAGES)
            .bind(from)
            .bind(current_month)
            .bind(today)
            .fetch_all(&self.pool)
            .await?)
    }
}

pub fn parse_month(month: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .map_err(|_| AppError::Corrupted(format!("mês inválido '{month}'")))
}
