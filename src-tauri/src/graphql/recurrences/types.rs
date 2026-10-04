use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::graphql::transactions::types::EntryKind;
use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::FrequencyUnit")]
pub enum FrequencyUnit {
    Day,
    Week,
    Month,
    Year,
}

#[derive(SimpleObject)]
pub struct Recurrence {
    pub id: Uuid,
    pub account_id: Uuid,
    pub account_name: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub kind: EntryKind,
    pub amount: i64,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub unit: FrequencyUnit,
    pub interval: u32,
    pub paused_at: Option<DateTime<Utc>>,
    pub credit_id: Option<Uuid>,
    pub next_date: Option<NaiveDate>,
}

#[derive(InputObject)]
pub struct RecurrenceInput {
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub kind: EntryKind,
    pub amount: i64,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub unit: FrequencyUnit,
    #[graphql(default = 1)]
    pub interval: i64,
}

#[derive(SimpleObject)]
pub struct Occurrence {
    pub recurrence_id: Uuid,
    pub occurrence_date: NaiveDate,
    pub date: NaiveDate,
    pub amount: i64,
    pub kind: EntryKind,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub description: String,
    pub credit_id: Option<Uuid>,
    pub modified: bool,
}
