use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::errors::AppError;

const DISMISSED: &str = "SELECT key FROM dismissed_alerts";

const DISMISS: &str = "INSERT OR IGNORE INTO dismissed_alerts (key, dismissed_at) VALUES (?1, ?2)";

#[derive(Clone)]
pub struct AlertsRepository {
    pool: SqlitePool,
}

impl AlertsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn dismissed(&self) -> Result<HashSet<String>, AppError> {
        let keys: Vec<String> = sqlx::query_scalar(DISMISSED).fetch_all(&self.pool).await?;
        Ok(keys.into_iter().collect())
    }

    pub async fn dismiss(&self, key: &str, now: DateTime<Utc>) -> Result<(), AppError> {
        sqlx::query(DISMISS)
            .bind(key)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
