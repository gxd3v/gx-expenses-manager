use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, Connection};

use crate::errors::AppError;

const SQLITE_NOTADB: &str = "26";
const CACHE_SIZE_KIB: &str = "-32768";
const MAX_CONNECTIONS: u32 = 4;
const SHARING_VIOLATION: i32 = 32;
const FILE_RETRIES: u32 = 50;
const RELEASE_RETRIES: u32 = 50;
const CHECKPOINT: &str = "PRAGMA wal_checkpoint(TRUNCATE)";
const FILE_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(200);
const MIGRATIONS_TABLE: &str =
    "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE name = '_sqlx_migrations')";
const APPLIED: &str = "SELECT version FROM _sqlx_migrations WHERE success = 1";
const FOREIGN_KEY_CHECK: &str = "PRAGMA foreign_key_check";

pub async fn open(path: &Path, password: &str, backup_dir: &Path) -> Result<SqlitePool, AppError> {
    if password.is_empty() {
        return Err(AppError::validation("a password é obrigatória"));
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect_with(
            options(path, password)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal),
        )
        .await
        .map_err(wrong_password)?;

    migrate(&pool, options(path, password), backup_dir).await?;
    Ok(pool)
}

pub async fn verify(path: &Path, password: &str) -> Result<(), AppError> {
    let mut conn = options(path, password)
        .read_only(true)
        .connect()
        .await
        .map_err(wrong_password)?;
    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&mut conn)
        .await
        .map_err(wrong_password)?;
    conn.close().await?;

    if result != "ok" {
        return Err(AppError::Corrupted(result));
    }
    Ok(())
}

pub async fn rekey(path: &Path, password: &str, new_password: &str) -> Result<(), AppError> {
    if new_password.len() < 8 {
        return Err(AppError::validation(
            "a nova password tem de ter pelo menos 8 caracteres",
        ));
    }
    let mut conn = options(path, password)
        .connect()
        .await
        .map_err(wrong_password)?;
    sqlx::query("PRAGMA journal_mode = DELETE")
        .execute(&mut conn)
        .await?;
    sqlx::query(&format!("PRAGMA rekey = {}", quote(new_password)))
        .execute(&mut conn)
        .await?;
    conn.close().await?;
    Ok(())
}

pub async fn close(pool: &SqlitePool) {
    if let Err(error) = sqlx::query(CHECKPOINT).execute(pool).await {
        log::warn!("checkpoint before close failed: {error}");
    }
    pool.close().await;
}

pub async fn snapshot(pool: &SqlitePool, path: &Path) -> Result<(), AppError> {
    sqlx::query("VACUUM INTO ?1")
        .bind(path.to_string_lossy())
        .execute(pool)
        .await?;
    Ok(())
}

pub fn discard(database: &Path) -> Result<(), AppError> {
    for path in [
        database.to_path_buf(),
        sidecar(database, "-wal"),
        sidecar(database, "-shm"),
    ] {
        retry(|| remove(&path))?;
    }
    Ok(())
}

pub fn replace(database: &Path, backup: &Path, safety_dir: &Path) -> Result<(), AppError> {
    set_aside(database, safety_dir, "pre-restore")?;
    retry(|| fs::copy(backup, database).map(drop))?;
    Ok(())
}

pub fn reset(database: &Path, safety_dir: &Path) -> Result<(), AppError> {
    set_aside(database, safety_dir, "pre-reset")?;
    retry(|| remove(database))?;
    Ok(())
}

fn set_aside(database: &Path, safety_dir: &Path, prefix: &str) -> std::io::Result<()> {
    wait_released(database);
    fs::create_dir_all(safety_dir)?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let safety = safety_dir.join(format!("expenses-manager-{prefix}-{stamp}.db"));
    retry(|| fs::copy(database, &safety).map(drop))?;

    for suffix in ["-wal", "-shm"] {
        let path = sidecar(database, suffix);
        retry(|| remove(&path))?;
    }
    Ok(())
}

fn remove(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

fn wait_released(database: &Path) {
    let wal = sidecar(database, "-wal");
    for _ in 0..RELEASE_RETRIES {
        if !wal.exists() {
            return;
        }
        std::thread::sleep(FILE_RETRY_DELAY);
    }
}

fn sidecar(database: &Path, suffix: &str) -> PathBuf {
    let mut path = database.as_os_str().to_owned();
    path.push(suffix);
    PathBuf::from(path)
}

fn retry(mut operation: impl FnMut() -> std::io::Result<()>) -> std::io::Result<()> {
    for _ in 1..FILE_RETRIES {
        match operation() {
            Err(error) if error.raw_os_error() == Some(SHARING_VIOLATION) => {
                std::thread::sleep(FILE_RETRY_DELAY)
            }
            result => return result,
        }
    }
    operation()
}

fn options(path: &Path, password: &str) -> SqliteConnectOptions {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .pragma("key", quote(password));
    #[cfg(test)]
    let options = options.pragma("kdf_iter", "1000");
    options
        .pragma("cache_size", CACHE_SIZE_KIB)
        .foreign_keys(true)
}

async fn migrate(
    pool: &SqlitePool,
    options: SqliteConnectOptions,
    backup_dir: &Path,
) -> Result<(), AppError> {
    let migrator = sqlx::migrate!();
    let has_table: bool = sqlx::query_scalar(MIGRATIONS_TABLE)
        .fetch_one(pool)
        .await
        .map_err(wrong_password)?;

    if has_table {
        let applied: Vec<i64> = sqlx::query_scalar(APPLIED).fetch_all(pool).await?;
        if migrator.iter().any(|m| !applied.contains(&m.version)) {
            fs::create_dir_all(backup_dir)?;
            let name = format!(
                "expenses-manager-pre-migration-{}.db",
                Utc::now().format("%Y%m%d-%H%M%S-%3f")
            );
            let path = backup_dir.join(name);
            sqlx::query("VACUUM INTO ?1")
                .bind(path.to_string_lossy())
                .execute(pool)
                .await?;
        }
    }

    let mut conn = options.foreign_keys(false).connect().await?;
    migrator.run_direct(&mut conn).await?;
    let violations = sqlx::query(FOREIGN_KEY_CHECK).fetch_all(&mut conn).await?;
    conn.close().await?;
    if !violations.is_empty() {
        return Err(AppError::Corrupted(
            "relações inválidas depois de atualizar a base de dados".into(),
        ));
    }
    Ok(())
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn wrong_password(err: sqlx::Error) -> AppError {
    let code = err.as_database_error().and_then(|e| e.code());
    match code.as_deref() {
        Some(SQLITE_NOTADB) => AppError::WrongPassword,
        _ => err.into(),
    }
}

#[cfg(test)]
pub mod testing {
    use std::path::PathBuf;

    use sqlx::SqlitePool;
    use uuid::Uuid;

    pub struct TestDatabase {
        pub dir: PathBuf,
        pub pool: SqlitePool,
    }

    impl TestDatabase {
        pub async fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("gx-test-{}", Uuid::now_v7()));
            std::fs::create_dir_all(&dir).unwrap();
            let pool = super::open(&dir.join("test.db"), "test-password", &dir.join("backups"))
                .await
                .unwrap();
            Self { dir, pool }
        }
    }

    impl Drop for TestDatabase {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
}
