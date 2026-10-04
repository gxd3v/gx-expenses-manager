use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::errors::AppError;
use crate::models::Transaction;
use crate::repositories::to_id;

#[derive(Debug, FromRow)]
pub struct TransactionRow {
    pub id: Hyphenated,
    pub account_id: Hyphenated,
    pub account_name: String,
    pub currency: String,
    pub category_id: Option<Hyphenated>,
    pub category_name: Option<String>,
    pub kind: String,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
    pub notes: Option<String>,
    pub confirmed: bool,
    pub transfer_id: Option<Hyphenated>,
    pub recurrence_id: Option<Hyphenated>,
    pub one_off: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub counterpart_account_id: Option<Hyphenated>,
    pub counterpart_account_name: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct TotalsRow {
    pub total_count: i64,
    pub income: i64,
    pub outcome: i64,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = AppError;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from(row.id),
            account_id: Uuid::from(row.account_id),
            account_name: row.account_name,
            currency: row.currency,
            category_id: to_id(row.category_id),
            category_name: row.category_name,
            kind: row.kind.parse()?,
            amount: row.amount,
            date: row.date,
            description: row.description,
            notes: row.notes,
            confirmed: row.confirmed,
            transfer_id: to_id(row.transfer_id),
            recurrence_id: to_id(row.recurrence_id),
            one_off: row.one_off,
            created_at: row.created_at,
            updated_at: row.updated_at,
            counterpart_account_id: to_id(row.counterpart_account_id),
            counterpart_account_name: row.counterpart_account_name,
        })
    }
}
