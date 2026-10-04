pub mod accounts;
pub mod alerts;
pub mod backups;
pub mod categories;
pub mod credits;
pub mod goals;
pub mod reconciliations;
pub mod recurrences;
pub mod reports;
pub mod saved_filters;
pub mod settings;
pub mod templates;
pub mod transactions;
pub mod transfers;

use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::errors::AppError;

fn affected(rows: u64) -> Result<(), AppError> {
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

fn opt_id(id: Option<Uuid>) -> Option<Hyphenated> {
    id.map(|id| id.hyphenated())
}

pub(crate) fn to_id(id: Option<Hyphenated>) -> Option<Uuid> {
    id.map(Uuid::from)
}
