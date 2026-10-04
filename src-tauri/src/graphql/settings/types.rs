use async_graphql::{Enum, InputObject, SimpleObject};

use crate::graphql::forecasts::types::ForecastMethod;
use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::DateFormat")]
pub enum DateFormat {
    DayMonthYear,
    YearMonthDay,
    MonthDayYear,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::Theme")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(SimpleObject, InputObject)]
#[graphql(input_name = "SettingsInput")]
pub struct Settings {
    pub currency: String,
    pub date_format: DateFormat,
    pub first_day_of_week: u8,
    pub theme: Theme,
    pub lock_timeout_minutes: u32,
    pub backup_dir: Option<String>,
    pub backup_frequency_days: u32,
    pub backup_keep: u32,
    pub forecast_method: ForecastMethod,
    pub forecast_history_months: u32,
    pub forecast_horizon_months: u32,
    pub notifications_enabled: bool,
    pub notify_days_ahead: u32,
    pub notify_upcoming: bool,
    pub notify_credits: bool,
    pub notify_goals: bool,
    pub notify_low_balance: bool,
    pub low_balance_threshold: i64,
    pub notify_negative_forecast: bool,
}
