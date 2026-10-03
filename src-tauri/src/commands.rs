use std::path::PathBuf;

use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::database;
use crate::errors::AppError;
use crate::graphql::{self, AppSchema};

const DATABASE_FILE: &str = "expenses.db";

struct Unlocked {
    pool: SqlitePool,
    schema: AppSchema,
}

#[derive(Default)]
pub struct Session(RwLock<Option<Unlocked>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    initialized: bool,
    unlocked: bool,
}

#[tauri::command]
pub async fn status(app: AppHandle, session: State<'_, Session>) -> Result<Status, AppError> {
    Ok(Status {
        initialized: database_path(&app)?.exists(),
        unlocked: session.0.read().await.is_some(),
    })
}

#[tauri::command]
pub async fn unlock(app: AppHandle, session: State<'_, Session>, password: String) -> Result<(), AppError> {
    let pool = database::open(&database_path(&app)?, &password).await?;
    let schema = graphql::schema(pool.clone());
    *session.0.write().await = Some(Unlocked { pool, schema });
    Ok(())
}

#[tauri::command]
pub async fn lock(session: State<'_, Session>) -> Result<(), AppError> {
    if let Some(unlocked) = session.0.write().await.take() {
        unlocked.pool.close().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn graphql(
    session: State<'_, Session>,
    request: async_graphql::Request,
) -> Result<async_graphql::Response, AppError> {
    let guard = session.0.read().await;
    let unlocked = guard.as_ref().ok_or(AppError::Locked)?;
    Ok(unlocked.schema.execute(request).await)
}

fn database_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir).map_err(tauri::Error::from)?;
    Ok(dir.join(DATABASE_FILE))
}
