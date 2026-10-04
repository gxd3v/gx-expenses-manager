use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::errors::AppError;
use crate::models::{Frequency, OccurrenceOverride, Recurrence};
use crate::repositories::to_id;

#[derive(Debug, FromRow)]
pub struct RecurrenceRow {
    pub id: Hyphenated,
    pub account_id: Hyphenated,
    pub account_name: String,
    pub category_id: Option<Hyphenated>,
    pub category_name: Option<String>,
    pub kind: String,
    pub amount: i64,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub unit: String,
    pub interval: i64,
    pub paused_at: Option<DateTime<Utc>>,
    pub credit_id: Option<Hyphenated>,
    pub to_account_id: Option<Hyphenated>,
    pub to_account_name: Option<String>,
    pub variable_amount: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct OverrideRow {
    pub recurrence_id: Hyphenated,
    pub occurrence_date: NaiveDate,
    pub status: String,
    pub amount: Option<i64>,
    pub date: Option<NaiveDate>,
    pub transaction_id: Option<Hyphenated>,
}

impl TryFrom<RecurrenceRow> for Recurrence {
    type Error = AppError;

    fn try_from(row: RecurrenceRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from(row.id),
            account_id: Uuid::from(row.account_id),
            account_name: row.account_name,
            category_id: to_id(row.category_id),
            category_name: row.category_name,
            kind: row.kind.parse()?,
            amount: row.amount,
            description: row.description,
            start_date: row.start_date,
            end_date: row.end_date,
            frequency: Frequency::new(row.unit.parse()?, row.interval)?,
            paused_at: row.paused_at,
            credit_id: to_id(row.credit_id),
            to_account_id: to_id(row.to_account_id),
            to_account_name: row.to_account_name,
            variable_amount: row.variable_amount,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<OverrideRow> for OccurrenceOverride {
    type Error = AppError;

    fn try_from(row: OverrideRow) -> Result<Self, Self::Error> {
        Ok(Self {
            recurrence_id: Uuid::from(row.recurrence_id),
            occurrence_date: row.occurrence_date,
            status: row.status.parse()?,
            amount: row.amount,
            date: row.date,
            transaction_id: to_id(row.transaction_id),
        })
    }
}
