use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use super::affected;
use crate::errors::AppError;
use crate::models::{WishlistInput, WishlistItem};

const SELECT: &str = "SELECT w.id, w.name, w.amount, w.account_id, a.name AS account_name, w.priority, w.notes, \
     w.purchased_at, w.created_at, w.updated_at \
     FROM wishlist_items w JOIN accounts a ON a.id = w.account_id ";

const ORDER: &str = "ORDER BY w.purchased_at IS NOT NULL, w.priority, w.created_at";

const INSERT: &str = "INSERT INTO wishlist_items (id, name, amount, account_id, priority, notes, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)";

const UPDATE: &str = "UPDATE wishlist_items SET name = ?2, amount = ?3, account_id = ?4, priority = ?5, notes = ?6, updated_at = ?7 \
     WHERE id = ?1";

const SET_PURCHASED: &str =
    "UPDATE wishlist_items SET purchased_at = ?2, updated_at = ?3 WHERE id = ?1";

const DELETE: &str = "DELETE FROM wishlist_items WHERE id = ?1";

#[derive(FromRow)]
struct WishlistRow {
    id: Hyphenated,
    name: String,
    amount: i64,
    account_id: Hyphenated,
    account_name: String,
    priority: u8,
    notes: Option<String>,
    purchased_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<WishlistRow> for WishlistItem {
    fn from(row: WishlistRow) -> Self {
        Self {
            id: Uuid::from(row.id),
            name: row.name,
            amount: row.amount,
            account_id: Uuid::from(row.account_id),
            account_name: row.account_name,
            priority: row.priority,
            notes: row.notes,
            purchased_at: row.purchased_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct WishlistRepository {
    pool: SqlitePool,
}

impl WishlistRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<WishlistItem>, AppError> {
        let rows: Vec<WishlistRow> = sqlx::query_as(&format!("{SELECT}{ORDER}"))
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(WishlistItem::from).collect())
    }

    pub async fn get(&self, id: Uuid) -> Result<WishlistItem, AppError> {
        let row: Option<WishlistRow> = sqlx::query_as(&format!("{SELECT}WHERE w.id = ?1"))
            .bind(id.hyphenated())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.ok_or(AppError::NotFound)?.into())
    }

    pub async fn insert(
        &self,
        id: Uuid,
        input: &WishlistInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(INSERT)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.amount)
            .bind(input.account_id.hyphenated())
            .bind(input.priority)
            .bind(&input.notes)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &WishlistInput,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(UPDATE)
            .bind(id.hyphenated())
            .bind(&input.name)
            .bind(input.amount)
            .bind(input.account_id.hyphenated())
            .bind(input.priority)
            .bind(&input.notes)
            .bind(now)
            .execute(&self.pool)
            .await?;
        affected(result.rows_affected())
    }

    pub async fn set_purchased(
        &self,
        id: Uuid,
        purchased_at: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query(SET_PURCHASED)
            .bind(id.hyphenated())
            .bind(purchased_at)
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
