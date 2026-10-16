use std::fs;
use std::path::PathBuf;

use age::secrecy::{ExposeSecret, SecretString};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::database;
use crate::errors::AppError;
use crate::graphql::{self, AppSchema};
use crate::joint;
use crate::models::dates::today;
use crate::module::AppContext;
use crate::profiles::{self, BACKUPS_DIR, DATABASE_FILE, Profile, SIMULATION_FILE};
use crate::repositories::recurrences::RecurrencesRepository;

const DATA_DIR_ENV: &str = "GX_DATA_DIR";
const MIN_PASSWORD_LENGTH: usize = 8;

struct Unlocked {
    pool: SqlitePool,
    schema: AppSchema,
    password: SecretString,
    simulation: bool,
    profile: Profile,
    dir: PathBuf,
    origin: Option<Origin>,
}

#[derive(Clone)]
struct Origin {
    profile: Profile,
    password: SecretString,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberRecurrence {
    description: String,
    kind: String,
    amount: i64,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    unit: String,
    interval: u32,
    category_name: Option<String>,
    account_name: String,
    variable_amount: bool,
    linked: bool,
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
    return_to: Option<Profile>,
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
        return_to: guard
            .as_ref()
            .and_then(|u| u.origin.as_ref())
            .map(|o| o.profile.clone()),
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
    let joints: Vec<Uuid> = profiles::list(&base)?
        .into_iter()
        .filter(|p| p.joint && p.members.contains(&user_id))
        .map(|p| p.id)
        .collect();
    if let Err(error) = joint::forget_others(&unlocked.pool, &joints).await {
        log::warn!("could not clean joint keys: {error}");
    }
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
    joint: bool,
) -> Result<(), AppError> {
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(AppError::validation(
            "a password tem de ter pelo menos 8 caracteres",
        ));
    }
    let base = data_dir(&app)?;
    lock_current(&session).await?;
    let profile = profiles::create(&base, &name, &color, joint)?;
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
pub async fn add_member(
    app: AppHandle,
    session: State<'_, Session>,
    user_id: Uuid,
    password: String,
) -> Result<Profile, AppError> {
    let base = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let unlocked = guard.as_mut().ok_or(AppError::Locked)?;
    if !unlocked.profile.joint || unlocked.simulation {
        return Err(AppError::validation(
            "só é possível numa conta conjunta, fora do modo simulação",
        ));
    }
    if profiles::get(&base, user_id)?.joint {
        return Err(AppError::validation(
            "uma conta conjunta não pode ser membro de outra",
        ));
    }

    let dir = profiles::dir(&base, user_id);
    let pool = database::open(&dir.join(DATABASE_FILE), &password, &dir.join(BACKUPS_DIR)).await?;
    let stored = joint::store(
        &pool,
        unlocked.profile.id,
        unlocked.password.expose_secret(),
    )
    .await;
    database::close(&pool).await;
    stored?;

    let mut members = unlocked.profile.members.clone();
    if !members.contains(&user_id) {
        members.push(user_id);
    }
    unlocked.profile = profiles::set_members(&base, unlocked.profile.id, members)?;
    Ok(unlocked.profile.clone())
}

#[tauri::command]
pub async fn remove_member(
    app: AppHandle,
    session: State<'_, Session>,
    user_id: Uuid,
) -> Result<Profile, AppError> {
    let base = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let unlocked = guard.as_mut().ok_or(AppError::Locked)?;
    let members = unlocked
        .profile
        .members
        .iter()
        .copied()
        .filter(|id| *id != user_id)
        .collect();
    unlocked.profile = profiles::set_members(&base, unlocked.profile.id, members)?;
    Ok(unlocked.profile.clone())
}

#[tauri::command]
pub async fn switch_to_joint(
    app: AppHandle,
    session: State<'_, Session>,
    joint_id: Uuid,
    password: Option<String>,
) -> Result<(), AppError> {
    let base = data_dir(&app)?;
    let target = profiles::get(&base, joint_id)?;
    let dir = profiles::dir(&base, joint_id);
    let mut guard = session.0.write().await;
    let unlocked = guard.take().ok_or(AppError::Locked)?;

    let key = match joint_key(&unlocked, &target, &dir, password).await {
        Ok(key) => key,
        Err(error) => {
            *guard = Some(unlocked);
            return Err(error);
        }
    };

    database::close(&unlocked.pool).await;
    match open_session(target, dir, key, false).await {
        Ok(mut joint) => {
            joint.origin = Some(Origin {
                profile: unlocked.profile.clone(),
                password: unlocked.password.clone(),
            });
            *guard = Some(joint);
            Ok(())
        }
        Err(error) => {
            let password = unlocked.password.expose_secret().to_string();
            *guard = Some(reopen(&unlocked, password, false).await?);
            Err(error)
        }
    }
}

async fn joint_key(
    unlocked: &Unlocked,
    target: &Profile,
    dir: &std::path::Path,
    password: Option<String>,
) -> Result<String, AppError> {
    if unlocked.simulation || !target.joint || !target.members.contains(&unlocked.profile.id) {
        return Err(AppError::validation(
            "não é possível trocar para esta conta conjunta",
        ));
    }
    let key = match password {
        Some(password) => password,
        None => joint::key(&unlocked.pool, target.id)
            .await?
            .ok_or_else(|| AppError::validation("é necessária a password da conta conjunta"))?,
    };
    database::verify(&dir.join(DATABASE_FILE), &key).await?;
    joint::store(&unlocked.pool, target.id, &key).await?;
    Ok(key)
}

#[tauri::command]
pub async fn switch_back(app: AppHandle, session: State<'_, Session>) -> Result<(), AppError> {
    let base = data_dir(&app)?;
    let mut guard = session.0.write().await;
    let origin = guard
        .as_ref()
        .and_then(|u| u.origin.clone())
        .ok_or_else(|| AppError::validation("não há sessão anterior"))?;
    lock_guarded(&mut guard).await?;

    let profile = profiles::get(&base, origin.profile.id).unwrap_or(origin.profile);
    let dir = profiles::dir(&base, profile.id);
    let password = origin.password.expose_secret().to_string();
    *guard = Some(open_session(profile, dir, password, false).await?);
    Ok(())
}

#[tauri::command]
pub async fn member_recurrences(
    app: AppHandle,
    session: State<'_, Session>,
    user_id: Uuid,
    password: Option<String>,
) -> Result<Vec<MemberRecurrence>, AppError> {
    let base = data_dir(&app)?;
    let guard = session.0.read().await;
    let unlocked = guard.as_ref().ok_or(AppError::Locked)?;
    if !unlocked.profile.members.contains(&user_id) {
        return Err(AppError::validation(
            "o utilizador não é membro desta conta",
        ));
    }
    let remembered = unlocked
        .origin
        .as_ref()
        .filter(|o| o.profile.id == user_id)
        .map(|o| o.password.expose_secret().to_string());
    let password = password
        .or(remembered)
        .ok_or_else(|| AppError::validation("é necessária a password do membro"))?;

    let dir = profiles::dir(&base, user_id);
    let pool = database::open(&dir.join(DATABASE_FILE), &password, &dir.join(BACKUPS_DIR)).await?;
    let listed = RecurrencesRepository::new(pool.clone()).list().await;
    database::close(&pool).await;

    let today = today();
    Ok(listed?
        .into_iter()
        .filter(|r| r.end_date.is_none_or(|end| end >= today))
        .map(|r| MemberRecurrence {
            linked: r.credit_id.is_some() || r.to_account_id.is_some(),
            description: r.description,
            kind: r.kind.as_str().to_uppercase(),
            amount: r.amount,
            start_date: r.start_date,
            end_date: r.end_date,
            unit: r.frequency.unit.as_str().to_uppercase(),
            interval: r.frequency.interval,
            category_name: r.category_name,
            account_name: r.account_name,
            variable_amount: r.variable_amount,
        })
        .collect())
}

#[tauri::command]
pub async fn lock(session: State<'_, Session>) -> Result<(), AppError> {
    lock_current(&session).await
}

async fn lock_current(session: &Session) -> Result<(), AppError> {
    lock_guarded(&mut *session.0.write().await).await
}

async fn lock_guarded(guard: &mut Option<Unlocked>) -> Result<(), AppError> {
    if let Some(unlocked) = guard.take() {
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
    let mut session = open_session(profile, unlocked.dir.clone(), password, simulation).await?;
    session.origin = unlocked.origin.clone();
    Ok(session)
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
        origin: None,
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
