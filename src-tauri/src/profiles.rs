use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database;
use crate::errors::AppError;

pub const DATABASE_FILE: &str = "expenses.db";
pub const SIMULATION_FILE: &str = "simulation.db";
pub const BACKUPS_DIR: &str = "backups";
const USERS_DIR: &str = "users";
const PROFILE_FILE: &str = "profile.json";
const LEGACY_BACKUP_PREFIX: &str = "expenses-manager-";
const MAX_NAME_LENGTH: usize = 40;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub joint: bool,
    #[serde(default)]
    pub members: Vec<Uuid>,
}

impl Profile {
    pub fn backup_tag(id: Uuid) -> String {
        format!("u{}", &id.simple().to_string()[24..])
    }
}

pub fn dir(base: &Path, id: Uuid) -> PathBuf {
    base.join(USERS_DIR).join(id.hyphenated().to_string())
}

pub fn list(base: &Path) -> Result<Vec<Profile>, AppError> {
    let users = base.join(USERS_DIR);
    if !users.exists() {
        return Ok(Vec::new());
    }

    let mut profiles: Vec<Profile> = fs::read_dir(users)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| read(&entry.path().join(PROFILE_FILE)))
        .collect();
    profiles.sort_by_key(|p| p.created_at);
    Ok(profiles)
}

pub fn get(base: &Path, id: Uuid) -> Result<Profile, AppError> {
    read(&dir(base, id).join(PROFILE_FILE)).ok_or(AppError::NotFound)
}

pub fn has_legacy(base: &Path) -> bool {
    base.join(DATABASE_FILE).exists()
}

pub fn create(base: &Path, name: &str, color: &str, joint: bool) -> Result<Profile, AppError> {
    let profile = Profile {
        id: Uuid::now_v7(),
        name: validate(base, None, name, color)?,
        color: color.to_string(),
        created_at: Utc::now(),
        joint,
        members: Vec::new(),
    };
    fs::create_dir_all(dir(base, profile.id))?;
    write(base, &profile)?;
    Ok(profile)
}

pub fn update(base: &Path, id: Uuid, name: &str, color: &str) -> Result<Profile, AppError> {
    let profile = Profile {
        name: validate(base, Some(id), name, color)?,
        color: color.to_string(),
        ..get(base, id)?
    };
    write(base, &profile)?;
    Ok(profile)
}

pub fn set_members(base: &Path, id: Uuid, members: Vec<Uuid>) -> Result<Profile, AppError> {
    let profile = Profile {
        members,
        ..get(base, id)?
    };
    write(base, &profile)?;
    Ok(profile)
}

pub fn remove(base: &Path, id: Uuid) -> Result<(), AppError> {
    let path = dir(base, id);
    database::retry(|| fs::remove_dir_all(&path))?;
    Ok(())
}

pub fn discard(base: &Path, id: Uuid) {
    if let Err(error) = remove(base, id) {
        log::warn!("could not remove an unfinished user folder: {error}");
    }
}

pub fn migrate_legacy(base: &Path, name: &str, color: &str) -> Result<Profile, AppError> {
    if !has_legacy(base) {
        return Err(AppError::NotFound);
    }
    let backups = base.join(BACKUPS_DIR);
    fs::create_dir_all(&backups)?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let safety = backups.join(format!("{LEGACY_BACKUP_PREFIX}pre-users-{stamp}.db"));
    database::retry(|| fs::copy(base.join(DATABASE_FILE), &safety).map(drop))?;

    let profile = create(base, name, color, false)?;
    let target = dir(base, profile.id);
    let database = base.join(DATABASE_FILE);
    let moves: Vec<(PathBuf, PathBuf)> = [
        database.clone(),
        database::sidecar(&database, "-wal"),
        database::sidecar(&database, "-shm"),
        backups,
    ]
    .into_iter()
    .filter(|path| path.exists())
    .map(|path| {
        let to = target.join(path.file_name().unwrap_or_default());
        (path, to)
    })
    .collect();

    let mut done = Vec::new();
    for (from, to) in &moves {
        if let Err(error) = database::retry(|| fs::rename(from, to)) {
            for (from, to) in done.iter().rev() {
                let _ = fs::rename(to, from);
            }
            discard(base, profile.id);
            return Err(error.into());
        }
        done.push((from.clone(), to.clone()));
    }

    database::discard(&base.join(SIMULATION_FILE))?;
    tag_backups(&target.join(BACKUPS_DIR), &Profile::backup_tag(profile.id));
    Ok(profile)
}

fn tag_backups(dir: &Path, tag: &str) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
        let Some(rest) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_prefix(LEGACY_BACKUP_PREFIX))
        else {
            continue;
        };
        let tagged = dir.join(format!("{LEGACY_BACKUP_PREFIX}{tag}-{rest}"));
        if let Err(error) = fs::rename(&path, tagged) {
            log::warn!("could not tag a backup: {error}");
        }
    }
}

fn validate(base: &Path, id: Option<Uuid>, name: &str, color: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH {
        return Err(AppError::validation(
            "o nome tem de ter entre 1 e 40 caracteres",
        ));
    }
    let valid_color = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if !valid_color {
        return Err(AppError::validation("cor inválida"));
    }
    let taken = list(base)?
        .iter()
        .any(|p| Some(p.id) != id && p.name.to_lowercase() == name.to_lowercase());
    if taken {
        return Err(AppError::conflict("já existe um utilizador com esse nome"));
    }
    Ok(name.to_string())
}

fn read(path: &Path) -> Option<Profile> {
    let bytes = fs::read(path).ok()?;
    match serde_json::from_slice(&bytes) {
        Ok(profile) => Some(profile),
        Err(error) => {
            log::warn!("ignoring an invalid user profile: {error}");
            None
        }
    }
}

fn write(base: &Path, profile: &Profile) -> Result<(), AppError> {
    let target = dir(base, profile.id).join(PROFILE_FILE);
    let temporary = target.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(profile)
        .map_err(|error| AppError::Corrupted(error.to_string()))?;
    fs::write(&temporary, json)?;
    database::retry(|| fs::rename(&temporary, &target))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "test-password";
    const COLOR: &str = "#BFDDF7";

    struct Base(PathBuf);

    impl Base {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("gx-profiles-{}", Uuid::now_v7()));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Base {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn users_are_listed_in_creation_order_with_unique_names() {
        let base = Base::new();
        let ana = create(&base.0, " Ana ", COLOR, false).unwrap();
        let rui = create(&base.0, "Rui", "#F8C8DC", false).unwrap();

        assert_eq!(ana.name, "Ana");
        assert_eq!(list(&base.0).unwrap(), vec![ana.clone(), rui.clone()]);
        assert!(create(&base.0, "ana", COLOR, false).is_err());
        assert!(create(&base.0, "", COLOR, false).is_err());
        assert!(create(&base.0, "Maria", "azul", false).is_err());
        assert!(update(&base.0, rui.id, "Ana", COLOR).is_err());

        let renamed = update(&base.0, rui.id, "Rui Pedro", COLOR).unwrap();
        assert_eq!(get(&base.0, rui.id).unwrap(), renamed);

        remove(&base.0, ana.id).unwrap();
        assert_eq!(list(&base.0).unwrap(), vec![renamed]);
    }

    #[tokio::test]
    async fn legacy_install_moves_into_its_own_user() {
        let base = Base::new();
        let legacy = base.0.join(DATABASE_FILE);
        let pool = database::open(&legacy, PASSWORD, &base.0.join(BACKUPS_DIR))
            .await
            .unwrap();
        database::close(&pool).await;
        fs::create_dir_all(base.0.join(BACKUPS_DIR)).unwrap();
        fs::write(
            base.0
                .join(BACKUPS_DIR)
                .join("expenses-manager-auto-20261001.db"),
            b"",
        )
        .unwrap();
        fs::write(base.0.join(SIMULATION_FILE), b"").unwrap();

        let profile = migrate_legacy(&base.0, "Gustavo", COLOR).unwrap();
        let target = dir(&base.0, profile.id);
        let tag = Profile::backup_tag(profile.id);

        assert!(!has_legacy(&base.0));
        assert!(!base.0.join(BACKUPS_DIR).exists());
        assert!(!base.0.join(SIMULATION_FILE).exists());
        database::verify(&target.join(DATABASE_FILE), PASSWORD)
            .await
            .unwrap();
        let backups: Vec<String> = fs::read_dir(target.join(BACKUPS_DIR))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(backups.len(), 2);
        assert!(
            backups
                .iter()
                .all(|name| name.starts_with(&format!("expenses-manager-{tag}-")))
        );
        assert!(backups.iter().any(|name| name.contains("-pre-users-")));
        assert_eq!(list(&base.0).unwrap(), vec![profile]);
    }

    #[test]
    fn joint_users_keep_their_members() {
        let base = Base::new();
        let ana = create(&base.0, "Ana", COLOR, false).unwrap();
        let casa = create(&base.0, "Casa", COLOR, true).unwrap();
        assert!(casa.joint && casa.members.is_empty());

        set_members(&base.0, casa.id, vec![ana.id]).unwrap();
        let renamed = update(&base.0, casa.id, "Casa Nova", "#F8C8DC").unwrap();
        assert!(renamed.joint);
        assert_eq!(renamed.members, vec![ana.id]);
    }

    #[test]
    fn profiles_written_before_joint_users_still_load() {
        let base = Base::new();
        let id = Uuid::now_v7();
        fs::create_dir_all(dir(&base.0, id)).unwrap();
        let json = format!(
            r##"{{"id":"{id}","name":"Ana","color":"#BFDDF7","createdAt":"2026-10-10T10:00:00Z"}}"##
        );
        fs::write(dir(&base.0, id).join(PROFILE_FILE), json).unwrap();

        let profile = get(&base.0, id).unwrap();
        assert!(!profile.joint && profile.members.is_empty());
    }

    #[test]
    fn migration_needs_a_legacy_database() {
        let base = Base::new();
        assert!(migrate_legacy(&base.0, "Gustavo", COLOR).is_err());
        assert!(list(&base.0).unwrap().is_empty());
    }
}
