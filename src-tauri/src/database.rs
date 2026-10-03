use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use crate::errors::AppError;

const SQLITE_NOTADB: &str = "26";

pub async fn open(path: &Path, password: &str) -> Result<SqlitePool, AppError> {
    if password.is_empty() {
        return Err(AppError::Validation("password is required".into()));
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .pragma("key", quote(password))
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .map_err(wrong_password)?;

    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
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
