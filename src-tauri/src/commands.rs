use std::fs;
use std::path::PathBuf;

use age::secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::database;
use crate::errors::AppError;
use crate::graphql::{self, AppSchema};
use crate::module::AppContext;
use crate::profiles::{self, BACKUPS_DIR, DATABASE_FILE, Profile, SIMULATION_FILE};

const DATA_DIR_ENV: &str = "GX_DATA_DIR";
const MIN_PASSWORD_LENGTH: usize = 8;

struct Unlocked {
    pool: SqlitePool,
    schema: AppSchema,
    password: SecretString,
    simulation: bool,
    profile: Profile,
    dir: PathBuf,
}

#[derive(Default)]
pub struct Session(RwLock<Option<Unlocked>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    users: Vec<Profile>,
    legacy: bool,
    unlocked: bool,
    user: Option<Profile>,
    simulation: bool,
    data_dir: String,
}

#[tauri::command]
pub async fn status(app: AppHandle, session: State<'_, Session>) -> Result<Status, AppError> {
    let base = data_dir(&app)?;
    let guard = session.0.read().await;
    Ok(Status {
        users: profiles::list(&base)?,
        legacy: profiles::has_legacy(&base),
        unlocked: guard.is_some(),
        user: guard.as_ref().map(|u| u.profile.clone()),
        simulation: guard.as_ref().is_some_and(|u| u.simulation),
        data_dir: guard
            .as_ref()
            .map_or(base, |u| u.dir.clone())
            .to_string_lossy()
            .into_owned(),
    })
}

#[tauri::command]
pub async fn unlock(
    app: AppHandle,
    session: State<'_, Session>,
    user_id: Uuid,
    password: String,
) -> Result<(), AppError> {
    let base = data_dir(&app)?;
    let profile = profiles::get(&base, user_id)?;
    let dir = profiles::dir(&base, user_id);
    if !dir.join(DATABASE_FILE).exists() {
        return Err(AppError::NotFound);
    }
    lock_current(&session).await?;
    database::discard(&dir.join(SIMULATION_FILE))?;
    let unlocked = open_session(profile, dir, password, false).await?;
    *session.0.write().await = Some(unlocked);
    Ok(())
}

#[tauri::command]
pub async fn create_user(
    app: AppHandle,
    session: State<'_, Session>,
    name: String,
    color: String,
    password: String,
) -> Result<(), AppError> {
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(AppError::validation(
            "a password tem de ter pelo menos 8 caracteres",
        ));
    }
    let base = data_dir(&app)?;
    lock_current(&session).await?;
    let profile = profiles::create(&base, &name, &color)?;
    let dir = profiles::dir(&base, profile.id);
    match open_session(profile.clone(), dir, password, false).await {
        Ok(unlocked) => {
            *session.0.write().await = Some(unlocked);
            Ok(())
        }
        Err(error) => {
            profiles::discard(&base, profile.id);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn migrate_user(
    app: AppHandle,
    session: State<'_, Session>,
    name: String,
    color: String,
    password: String,
) -> Result<(), AppError> {
    let base = data_dir(&app)?;
    database::verify(&base.join(DATABASE_FILE), &password).await?;
    lock_current(&session).await?;
    let profile = profiles::migrate_legacy(&base, &name, &color)?;
    let dir = profiles::dir(&base, profile.id);
    let unlocked = open_session(profile, dir, password, false).await?;
    *session.0.write().await = Some(unlocked);
    Ok(())
}

#[tauri::command]
pub async fn update_user(
    app: AppHandle,
    session: State<'_, Session>,
    name: String,
    color: String,
) -> Result<Profile, AppError> {
    let base = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let unlocked = guard.as_mut().ok_or(AppError::Locked)?;
    let profile = profiles::update(&base, unlocked.profile.id, &name, &color)?;
    unlocked.profile = profile.clone();
    Ok(profile)
}

#[tauri::command]
pub async fn delete_user(
    app: AppHandle,
    session: State<'_, Session>,
    password: String,
) -> Result<(), AppError> {
    let base = data_dir(&app)?;
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
    profiles::remove(&base, unlocked.profile.id)
}

#[tauri::command]
pub async fn lock(session: State<'_, Session>) -> Result<(), AppError> {
    lock_current(&session).await
}

async fn lock_current(session: &Session) -> Result<(), AppError> {
    if let Some(unlocked) = session.0.write().await.take() {
        database::close(&unlocked.pool).await;
        if unlocked.simulation {
            database::discard(&unlocked.dir.join(SIMULATION_FILE))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn start_simulation(session: State<'_, Session>) -> Result<(), AppError> {
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;
    if unlocked.simulation {
        *guard = Some(unlocked);
        return Ok(());
    }

    let password = unlocked.password.expose_secret().to_string();
    let copy = unlocked.dir.join(SIMULATION_FILE);
    let simulated = match database::discard(&copy) {
        Ok(()) => match database::snapshot(&unlocked.pool, &copy).await {
            Ok(()) => reopen(&unlocked, password, true).await,
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
pub async fn stop_simulation(session: State<'_, Session>) -> Result<(), AppError> {
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;
    if !unlocked.simulation {
        *guard = Some(unlocked);
        return Ok(());
    }

    database::close(&unlocked.pool).await;
    let password = unlocked.password.expose_secret().to_string();
    *guard = Some(reopen(&unlocked, password, false).await?);
    database::discard(&unlocked.dir.join(SIMULATION_FILE))
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
    session: State<'_, Session>,
    current: String,
    new_password: String,
) -> Result<(), AppError> {
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
    let database = unlocked.dir.join(DATABASE_FILE);
    let password = match database::rekey(&database, &current, &new_password).await {
        Ok(()) => new_password,
        Err(error) => {
            *guard = Some(reopen(&unlocked, current, false).await?);
            return Err(error);
        }
    };
    *guard = Some(reopen(&unlocked, password, false).await?);
    Ok(())
}

#[tauri::command]
pub async fn restore_backup(
    session: State<'_, Session>,
    path: String,
    password: Option<String>,
) -> Result<(), AppError> {
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
        &unlocked.dir.join(DATABASE_FILE),
        &backup,
        &unlocked.dir.join(BACKUPS_DIR),
    )?;
    *guard = Some(reopen(&unlocked, password, false).await?);
    Ok(())
}

#[tauri::command]
pub async fn reset_data(session: State<'_, Session>, password: String) -> Result<(), AppError> {
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
    database::reset(
        &unlocked.dir.join(DATABASE_FILE),
        &unlocked.dir.join(BACKUPS_DIR),
    )?;
    *guard = Some(reopen(&unlocked, password, false).await?);
    Ok(())
}

async fn reopen(
    unlocked: &Unlocked,
    password: String,
    simulation: bool,
) -> Result<Unlocked, AppError> {
    let profile = unlocked.profile.clone();
    open_session(profile, unlocked.dir.clone(), password, simulation).await
}

async fn open_session(
    profile: Profile,
    dir: PathBuf,
    password: String,
    simulation: bool,
) -> Result<Unlocked, AppError> {
    let file = match simulation {
        true => SIMULATION_FILE,
        false => DATABASE_FILE,
    };
    let pool = database::open(&dir.join(file), &password, &dir.join(BACKUPS_DIR)).await?;
    let context = AppContext {
        data_dir: dir.clone(),
        password: SecretString::from(password.clone()),
        simulation,
        user: profile.id,
    };
    Ok(Unlocked {
        schema: graphql::schema(pool.clone(), context),
        pool,
        password: SecretString::from(password),
        simulation,
        profile,
        dir,
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
