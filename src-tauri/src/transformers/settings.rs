use crate::graphql::settings::types;
use crate::models;

impl From<models::Settings> for types::Settings {
    fn from(settings: models::Settings) -> Self {
        Self {
            currency: settings.currency,
            date_format: settings.date_format.into(),
            first_day_of_week: settings.first_day_of_week,
            theme: settings.theme.into(),
            lock_timeout_minutes: settings.lock_timeout_minutes,
            backup_dir: settings.backup_dir,
            backup_frequency_days: settings.backup_frequency_days,
            backup_keep: settings.backup_keep,
            forecast_method: settings.forecast_method.into(),
            forecast_history_months: settings.forecast_history_months,
            forecast_horizon_months: settings.forecast_horizon_months,
            notifications_enabled: settings.notifications_enabled,
            notify_days_ahead: settings.notify_days_ahead,
            notify_upcoming: settings.notify_upcoming,
            notify_credits: settings.notify_credits,
            notify_goals: settings.notify_goals,
            notify_low_balance: settings.notify_low_balance,
            low_balance_threshold: settings.low_balance_threshold,
            notify_negative_forecast: settings.notify_negative_forecast,
        }
    }
}

impl From<types::Settings> for models::Settings {
    fn from(settings: types::Settings) -> Self {
        Self {
            currency: settings.currency,
            date_format: settings.date_format.into(),
            first_day_of_week: settings.first_day_of_week,
            theme: settings.theme.into(),
            lock_timeout_minutes: settings.lock_timeout_minutes,
            backup_dir: settings.backup_dir,
            backup_frequency_days: settings.backup_frequency_days,
            backup_keep: settings.backup_keep,
            forecast_method: settings.forecast_method.into(),
            forecast_history_months: settings.forecast_history_months,
            forecast_horizon_months: settings.forecast_horizon_months,
            notifications_enabled: settings.notifications_enabled,
            notify_days_ahead: settings.notify_days_ahead,
            notify_upcoming: settings.notify_upcoming,
            notify_credits: settings.notify_credits,
            notify_goals: settings.notify_goals,
            notify_low_balance: settings.notify_low_balance,
            low_balance_threshold: settings.low_balance_threshold,
            notify_negative_forecast: settings.notify_negative_forecast,
        }
    }
}
