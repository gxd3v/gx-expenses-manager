use serde::{Deserialize, Serialize};

use super::ForecastMethod;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateFormat {
    DayMonthYear,
    YearMonthDay,
    MonthDayYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    pub check_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            currency: "EUR".into(),
            date_format: DateFormat::DayMonthYear,
            first_day_of_week: 1,
            theme: Theme::System,
            lock_timeout_minutes: 10,
            backup_dir: None,
            backup_frequency_days: 7,
            backup_keep: 10,
            forecast_method: ForecastMethod::Recurring,
            forecast_history_months: 6,
            forecast_horizon_months: 6,
            notifications_enabled: true,
            notify_days_ahead: 3,
            notify_upcoming: true,
            notify_credits: true,
            notify_goals: true,
            notify_low_balance: true,
            low_balance_threshold: 10_000,
            notify_negative_forecast: true,
            check_updates: true,
        }
    }
}
