use std::fs;
use std::path::{Path, PathBuf};

use age::secrecy::{ExposeSecret, SecretString};
use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::database;
use crate::errors::AppError;
use crate::graphql::{self, AppSchema};
use crate::module::AppContext;

const DATABASE_FILE: &str = "expenses.db";

struct Unlocked {
    pool: SqlitePool,
    schema: AppSchema,
    password: SecretString,
}

#[derive(Default)]
pub struct Session(RwLock<Option<Unlocked>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    initialized: bool,
    unlocked: bool,
    data_dir: String,
}

#[tauri::command]
pub async fn status(app: AppHandle, session: State<'_, Session>) -> Result<Status, AppError> {
    let data_dir = data_dir(&app)?;
    Ok(Status {
        initialized: data_dir.join(DATABASE_FILE).exists(),
        unlocked: session.0.read().await.is_some(),
        data_dir: data_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub async fn unlock(
    app: AppHandle,
    session: State<'_, Session>,
    password: String,
) -> Result<(), AppError> {
    let unlocked = open_session(&data_dir(&app)?, password).await?;
    *session.0.write().await = Some(unlocked);
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
    let response = unlocked.schema.execute(request).await;
    for error in &response.errors {
        log::warn!("graphql error: {}", error.message);
    }
    Ok(response)
}

#[tauri::command]
pub async fn change_password(
    app: AppHandle,
    session: State<'_, Session>,
    current: String,
    new_password: String,
) -> Result<(), AppError> {
    let data_dir = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;
    if unlocked.password.expose_secret() != current {
        *guard = Some(unlocked);
        return Err(AppError::WrongPassword);
    }

    unlocked.pool.close().await;
    let password =
        match database::rekey(&data_dir.join(DATABASE_FILE), &current, &new_password).await {
            Ok(()) => new_password,
            Err(error) => {
                *guard = Some(open_session(&data_dir, current).await?);
                return Err(error);
            }
        };
    *guard = Some(open_session(&data_dir, password).await?);
    Ok(())
}

#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    session: State<'_, Session>,
    path: String,
    password: Option<String>,
) -> Result<(), AppError> {
    let data_dir = data_dir(&app)?;
    let backup = PathBuf::from(path);
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;
    let password = password.unwrap_or_else(|| unlocked.password.expose_secret().to_string());

    if let Err(error) = database::verify(&backup, &password).await {
        *guard = Some(unlocked);
        return Err(error);
    }

    unlocked.pool.close().await;
    replace_database(&data_dir, &backup)?;
    *guard = Some(open_session(&data_dir, password).await?);
    Ok(())
}

async fn open_session(data_dir: &Path, password: String) -> Result<Unlocked, AppError> {
    let pool = database::open(
        &data_dir.join(DATABASE_FILE),
        &password,
        &data_dir.join("backups"),
    )
    .await?;
    let context = AppContext {
        data_dir: data_dir.to_path_buf(),
        password: SecretString::from(password.clone()),
    };
    Ok(Unlocked {
        schema: graphql::schema(pool.clone(), context),
        pool,
        password: SecretString::from(password),
    })
}

fn replace_database(data_dir: &Path, backup: &Path) -> Result<(), AppError> {
    let database = data_dir.join(DATABASE_FILE);
    let safety_dir = data_dir.join("backups");
    fs::create_dir_all(&safety_dir)?;
    let safety = safety_dir.join(format!(
        "gx-expenses-pre-restore-{}.db",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    fs::copy(&database, safety)?;

    for suffix in ["-wal", "-shm"] {
        let file = data_dir.join(format!("{DATABASE_FILE}{suffix}"));
        if file.exists() {
            fs::remove_file(file)?;
        }
    }
    fs::copy(backup, &database)?;
    Ok(())
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
