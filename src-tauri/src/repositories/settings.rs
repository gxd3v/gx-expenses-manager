use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::Settings;

const GET: &str = "SELECT data FROM settings WHERE id = 1";

const UPSERT: &str = "INSERT INTO settings (id, data) VALUES (1, ?1) ON CONFLICT (id) DO UPDATE SET data = excluded.data";

#[derive(Clone)]
pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> Result<Settings, AppError> {
        let data: Option<String> = sqlx::query_scalar(GET).fetch_optional(&self.pool).await?;
        let Some(data) = data else {
            return Ok(Settings::default());
        };
        serde_json::from_str(&data).map_err(|e| AppError::Corrupted(e.to_string()))
    }

    pub async fn save(&self, settings: &Settings) -> Result<(), AppError> {
        let data =
            serde_json::to_string(settings).map_err(|e| AppError::Corrupted(e.to_string()))?;
        sqlx::query(UPSERT).bind(data).execute(&self.pool).await?;
        Ok(())
    }
}
