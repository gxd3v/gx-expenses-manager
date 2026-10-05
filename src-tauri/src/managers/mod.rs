pub mod accounts;
pub mod alerts;
pub mod backups;
pub mod categories;
pub mod credits;
pub mod forecasts;
pub mod goals;
pub mod purchases;
pub mod reconciliation;
pub mod recurrences;
pub mod reports;
pub mod search;
pub mod settings;
pub mod templates;
pub mod transactions;
pub mod transfers;

use chrono::{DateTime, Utc};

use crate::errors::AppError;

fn required(value: &str, message: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::validation(message));
    }
    Ok(value.to_string())
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn positive(amount: i64, message: &str) -> Result<(), AppError> {
    if amount <= 0 {
        return Err(AppError::validation(message));
    }
    Ok(())
}

fn archived_at(archived: bool) -> Option<DateTime<Utc>> {
    archived.then(Utc::now)
}

fn currency(value: &str) -> Result<String, AppError> {
    let currency = value.trim().to_uppercase();
    if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::validation(
            "a moeda tem de ser um código ISO de 3 letras",
        ));
    }
    Ok(currency)
}
