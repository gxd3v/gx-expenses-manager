use std::fs;
use std::path::Path;

use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, Connection};

use crate::errors::AppError;

const SQLITE_NOTADB: &str = "26";
const MIGRATIONS_TABLE: &str =
    "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE name = '_sqlx_migrations')";
const APPLIED: &str = "SELECT version FROM _sqlx_migrations WHERE success = 1";

pub async fn open(path: &Path, password: &str, backup_dir: &Path) -> Result<SqlitePool, AppError> {
    if password.is_empty() {
        return Err(AppError::validation("a password é obrigatória"));
    }

    let pool = SqlitePoolOptions::new()
        .connect_with(
            options(path, password)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal),
        )
        .await
        .map_err(wrong_password)?;

    migrate(&pool, backup_dir).await?;
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

fn options(path: &Path, password: &str) -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .filename(path)
        .pragma("key", quote(password))
        .foreign_keys(true)
}

async fn migrate(pool: &SqlitePool, backup_dir: &Path) -> Result<(), AppError> {
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
                "gx-expenses-pre-migration-{}.db",
                Utc::now().format("%Y%m%d-%H%M%S")
            );
            let path = backup_dir.join(name);
            sqlx::query("VACUUM INTO ?1")
                .bind(path.to_string_lossy())
                .execute(pool)
                .await?;
        }
    }

    migrator.run(pool).await?;
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
