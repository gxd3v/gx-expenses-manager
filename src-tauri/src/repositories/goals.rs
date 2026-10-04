use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use super::affected;
use crate::errors::AppError;
use crate::models::{Goal, GoalInput};

macro_rules! select {
    () => {
        "SELECT g.id, g.name, g.account_id, a.name AS account_name, g.target_amount, g.target_date, g.archived_at, \
         g.created_at, g.updated_at, \
         a.initial_balance + COALESCE((SELECT SUM(t.amount) FROM transactions t WHERE t.account_id = g.account_id AND t.date <= ?1), 0) \
         AS current_amount \
         FROM goals g JOIN accounts a ON a.id = g.account_id "
    };
}

const LIST: &str = concat!(
    select!(),
    "WHERE ?2 OR g.archived_at IS NULL ORDER BY g.archived_at IS NOT NULL, g.target_date IS NULL, g.target_date, g.name"
);

const GET: &str = concat!(select!(), "WHERE g.id = ?2");

const INSERT: &str = "INSERT INTO goals (id, name, account_id, target_amount, target_date, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)";

const UPDATE: &str = "UPDATE goals SET name = ?2, account_id = ?3, target_amount = ?4, target_date = ?5, updated_at = ?6 \
     WHERE id = ?1";

const SET_ARCHIVED: &str = "UPDATE goals SET archived_at = ?2, updated_at = ?3 WHERE id = ?1";

const DELETE: &str = "DELETE FROM goals WHERE id = ?1";

#[derive(FromRow)]
struct GoalRow {
    id: Hyphenated,
    name: String,
    account_id: Hyphenated,
    account_name: String,
    target_amount: i64,
    target_date: Option<NaiveDate>,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    current_amount: i64,
}

impl From<GoalRow> for Goal {
    fn from(row: GoalRow) -> Self {
        Self {
            id: Uuid::from(row.id),
            name: row.name,
            account_id: Uuid::from(row.account_id),
            account_name: row.account_name,
            target_amount: row.target_amount,
            target_date: row.target_date,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            current_amount: row.current_amount,
        }
    }
}

#[derive(Clone)]
pub struct GoalsRepository {
    pool: SqlitePool,
}

impl GoalsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        today: NaiveDate,
        include_archived: bool,
    ) -> Result<Vec<Goal>, AppError> {
        let rows: Vec<GoalRow> = sqlx::query_as(LIST)
            .bind(today)
            .bind(include_archived)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Goal::from).collect())
    }

    pub async fn get(&self, today: NaiveDate, id: Uuid) -> Result<Goal, AppError> {
        let row: Option<GoalRow> = sqlx::query_as(GET)
            .bind(today)
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.ok_or(AppError::NotFound)?.into())
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &GoalInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(INSERT)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.account_id.hyphenated())
            .bind(input.target_amount)
            .bind(input.target_date)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &GoalInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(UPDATE)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.account_id.hyphenated())
            .bind(input.target_amount)
            .bind(input.target_date)
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
        let result = sqlx::query(SET_ARCHIVED)
            .bind(id.hyphenated())
            .bind(archived_at)
            .bind(now)
            .execute(&self.pool)
            .await?;

        affected(result.rows_affected())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(DELETE)
            .bind(id.hyphenated())
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }
}
