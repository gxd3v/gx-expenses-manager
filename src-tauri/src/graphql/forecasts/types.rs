use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::ForecastMethod")]
pub enum ForecastMethod {
    Recurring,
    History,
}

#[derive(InputObject)]
pub struct AdjustmentInput {
    pub account_id: Uuid,
    pub to_account_id: Option<Uuid>,
    pub amount: i64,
    pub date: NaiveDate,
    #[graphql(default = 1)]
    pub repeat_months: u32,
}

#[derive(InputObject)]
pub struct RecurrenceChangeInput {
    pub recurrence_id: Uuid,
    pub amount: Option<i64>,
}

#[derive(InputObject)]
pub struct ForecastInput {
    pub months: u32,
    pub method: Option<ForecastMethod>,
    pub history_months: Option<u32>,
    #[graphql(default)]
    pub adjustments: Vec<AdjustmentInput>,
    #[graphql(default)]
    pub recurrence_changes: Vec<RecurrenceChangeInput>,
}

#[derive(SimpleObject)]
pub struct AccountBalance {
    pub account_id: Uuid,
    pub balance: i64,
}

#[derive(SimpleObject)]
pub struct ForecastMonth {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
    pub net: i64,
    pub total: i64,
    pub balances: Vec<AccountBalance>,
}

#[derive(SimpleObject)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub date: NaiveDate,
}

#[derive(SimpleObject)]
pub struct Forecast {
    pub months: Vec<ForecastMonth>,
    pub goals_reached: Vec<Milestone>,
    pub credits_paid: Vec<Milestone>,
}
