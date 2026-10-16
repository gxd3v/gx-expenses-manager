use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::AppError;

const GET: &str = "SELECT password FROM joint_keys WHERE profile_id = ?1";
const UPSERT: &str = "INSERT INTO joint_keys (profile_id, password) VALUES (?1, ?2) \
     ON CONFLICT (profile_id) DO UPDATE SET password = excluded.password";
const LIST: &str = "SELECT profile_id FROM joint_keys";
const DELETE: &str = "DELETE FROM joint_keys WHERE profile_id = ?1";

pub async fn key(pool: &SqlitePool, joint: Uuid) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar(GET)
        .bind(joint.hyphenated())
        .fetch_optional(pool)
        .await?)
}

pub async fn store(pool: &SqlitePool, joint: Uuid, password: &str) -> Result<(), AppError> {
    sqlx::query(UPSERT)
        .bind(joint.hyphenated())
        .bind(password)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn forget_others(pool: &SqlitePool, keep: &[Uuid]) -> Result<(), AppError> {
    let stored: Vec<String> = sqlx::query_scalar(LIST).fetch_all(pool).await?;
    let keep: Vec<String> = keep.iter().map(|id| id.hyphenated().to_string()).collect();
    for id in stored.iter().filter(|id| !keep.contains(id)) {
        sqlx::query(DELETE).bind(id).execute(pool).await?;
    }
    Ok(())
}
