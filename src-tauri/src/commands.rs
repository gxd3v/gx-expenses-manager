use std::fs;
use std::path::{Path, PathBuf};

use age::secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::database;
use crate::errors::AppError;
use crate::graphql::{self, AppSchema};
use crate::module::AppContext;

const DATABASE_FILE: &str = "expenses.db";
const SIMULATION_FILE: &str = "simulation.db";
const DATA_DIR_ENV: &str = "GX_DATA_DIR";

struct Unlocked {
    pool: SqlitePool,
    schema: AppSchema,
    password: SecretString,
    simulation: bool,
}

#[derive(Default)]
pub struct Session(RwLock<Option<Unlocked>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    initialized: bool,
    unlocked: bool,
    simulation: bool,
    data_dir: String,
}

#[tauri::command]
pub async fn status(app: AppHandle, session: State<'_, Session>) -> Result<Status, AppError> {
    let data_dir = data_dir(&app)?;
    let guard = session.0.read().await;
    Ok(Status {
        initialized: data_dir.join(DATABASE_FILE).exists(),
        unlocked: guard.is_some(),
        simulation: guard.as_ref().is_some_and(|u| u.simulation),
        data_dir: data_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub async fn unlock(
    app: AppHandle,
    session: State<'_, Session>,
    password: String,
) -> Result<(), AppError> {
    let data_dir = data_dir(&app)?;
    database::discard(&data_dir.join(SIMULATION_FILE))?;
    let unlocked = open_session(&data_dir, password, false).await?;
    *session.0.write().await = Some(unlocked);
    Ok(())
}

#[tauri::command]
pub async fn lock(app: AppHandle, session: State<'_, Session>) -> Result<(), AppError> {
    if let Some(unlocked) = session.0.write().await.take() {
        database::close(&unlocked.pool).await;
        if unlocked.simulation {
            database::discard(&data_dir(&app)?.join(SIMULATION_FILE))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn start_simulation(app: AppHandle, session: State<'_, Session>) -> Result<(), AppError> {
    let data_dir = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;
    if unlocked.simulation {
        *guard = Some(unlocked);
        return Ok(());
    }

    let password = unlocked.password.expose_secret().to_string();
    let copy = data_dir.join(SIMULATION_FILE);
    let simulated = match database::discard(&copy) {
        Ok(()) => match database::snapshot(&unlocked.pool, &copy).await {
            Ok(()) => open_session(&data_dir, password, true).await,
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    };
    match simulated {
        Ok(simulated) => {
            database::close(&unlocked.pool).await;
            *guard = Some(simulated);
            Ok(())
        }
        Err(error) => {
            *guard = Some(unlocked);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn stop_simulation(app: AppHandle, session: State<'_, Session>) -> Result<(), AppError> {
    let data_dir = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;
    if !unlocked.simulation {
        *guard = Some(unlocked);
        return Ok(());
    }

    database::close(&unlocked.pool).await;
    let password = unlocked.password.expose_secret().to_string();
    *guard = Some(open_session(&data_dir, password, false).await?);
    database::discard(&data_dir.join(SIMULATION_FILE))
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
    if unlocked.simulation {
        *guard = Some(unlocked);
        return Err(AppError::validation("indisponível em modo de simulação"));
    }
    if unlocked.password.expose_secret() != current {
        *guard = Some(unlocked);
        return Err(AppError::WrongPassword);
    }

    database::close(&unlocked.pool).await;
    let password =
        match database::rekey(&data_dir.join(DATABASE_FILE), &current, &new_password).await {
            Ok(()) => new_password,
            Err(error) => {
                *guard = Some(open_session(&data_dir, current, false).await?);
                return Err(error);
            }
        };
    *guard = Some(open_session(&data_dir, password, false).await?);
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
    if unlocked.simulation {
        *guard = Some(unlocked);
        return Err(AppError::validation("indisponível em modo de simulação"));
    }
    let password = password.unwrap_or_else(|| unlocked.password.expose_secret().to_string());

    if let Err(error) = database::verify(&backup, &password).await {
        *guard = Some(unlocked);
        return Err(error);
    }

    database::close(&unlocked.pool).await;
    database::replace(
        &data_dir.join(DATABASE_FILE),
        &backup,
        &data_dir.join("backups"),
    )?;
    *guard = Some(open_session(&data_dir, password, false).await?);
    Ok(())
}

#[tauri::command]
pub async fn reset_data(
    app: AppHandle,
    session: State<'_, Session>,
    password: String,
) -> Result<(), AppError> {
    let data_dir = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;
    if unlocked.simulation {
        *guard = Some(unlocked);
        return Err(AppError::validation("indisponível em modo de simulação"));
    }
    if unlocked.password.expose_secret() != password {
        *guard = Some(unlocked);
        return Err(AppError::WrongPassword);
    }

    database::close(&unlocked.pool).await;
    database::reset(&data_dir.join(DATABASE_FILE), &data_dir.join("backups"))?;
    *guard = Some(open_session(&data_dir, password, false).await?);
    Ok(())
}

async fn open_session(
    data_dir: &Path,
    password: String,
    simulation: bool,
) -> Result<Unlocked, AppError> {
    let file = match simulation {
        true => SIMULATION_FILE,
        false => DATABASE_FILE,
    };
    let pool = database::open(&data_dir.join(file), &password, &data_dir.join("backups")).await?;
    let context = AppContext {
        data_dir: data_dir.to_path_buf(),
        password: SecretString::from(password.clone()),
        simulation,
    };
    Ok(Unlocked {
        schema: graphql::schema(pool.clone(), context),
        pool,
        password: SecretString::from(password),
        simulation,
    })
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = match std::env::var_os(DATA_DIR_ENV) {
        Some(dir) => PathBuf::from(dir),
        None if cfg!(debug_assertions) => app.path().app_data_dir()?.join("dev"),
        None => app.path().app_data_dir()?,
    };
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
