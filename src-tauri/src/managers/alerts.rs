use chrono::{Days, NaiveDate, Utc};

use super::forecasts::{ForecastRequest, ForecastsManager};
use super::recurrences::RecurrencesManager;
use crate::errors::AppError;
use crate::models::{AccountKind, EntryKind, Settings};
use crate::repositories::accounts::AccountsRepository;
use crate::repositories::alerts::AlertsRepository;
use crate::repositories::goals::GoalsRepository;
use crate::repositories::settings::SettingsRepository;

const GOAL_WARNING_DAYS: i64 = 30;
const FORECAST_MONTHS: u32 = 3;

string_enum!(AlertKind {
    Upcoming => "upcoming",
    Credit => "credit",
    Goal => "goal",
    LowBalance => "low_balance",
    NegativeForecast => "negative_forecast",
});

#[derive(Debug, Clone)]
pub struct Alert {
    pub key: String,
    pub kind: AlertKind,
    pub title: String,
    pub message: String,
    pub date: Option<NaiveDate>,
}

#[derive(Clone)]
pub struct AlertsManager {
    repository: AlertsRepository,
    settings: SettingsRepository,
    recurrences: RecurrencesManager,
    accounts: AccountsRepository,
    goals: GoalsRepository,
    forecasts: ForecastsManager,
}

impl AlertsManager {
    pub fn new(
        repository: AlertsRepository,
        settings: SettingsRepository,
        recurrences: RecurrencesManager,
        accounts: AccountsRepository,
        goals: GoalsRepository,
        forecasts: ForecastsManager,
    ) -> Self {
        Self {
            repository,
            settings,
            recurrences,
            accounts,
            goals,
            forecasts,
        }
    }

    pub async fn alerts(&self, today: NaiveDate) -> Result<Vec<Alert>, AppError> {
        let settings = self.settings.get().await?;
        if !settings.notifications_enabled {
            return Ok(Vec::new());
        }

        let mut alerts = self.upcoming(today, &settings).await?;
        if settings.notify_upcoming {
            alerts.extend(self.ending(today, &settings).await?);
        }
        if settings.notify_goals {
            alerts.extend(self.goal_alerts(today).await?);
        }
        if settings.notify_low_balance {
            alerts.extend(
                self.low_balance(today, settings.low_balance_threshold)
                    .await?,
            );
        }
        if settings.notify_negative_forecast {
            alerts.extend(self.negative_forecast(today, &settings).await?);
        }
        let dismissed = self.repository.dismissed().await?;
        alerts.retain(|alert| !dismissed.contains(&alert.key));
        Ok(alerts)
    }

    pub async fn dismiss(&self, key: &str) -> Result<(), AppError> {
        self.repository.dismiss(key, Utc::now()).await
    }

    async fn upcoming(
        &self,
        today: NaiveDate,
        settings: &Settings,
    ) -> Result<Vec<Alert>, AppError> {
        let until = today
            .checked_add_days(Days::new(settings.notify_days_ahead.into()))
            .unwrap_or(today);
        let occurrences = self.recurrences.occurrences(today, until, None).await?;

        Ok(occurrences
            .into_iter()
            .filter_map(|o| {
                let (kind, title) = match (o.credit_id, o.kind) {
                    (Some(_), _) if settings.notify_credits => {
                        (AlertKind::Credit, "Prestação de crédito próxima")
                    }
                    _ if o.to_account_id.is_some() && settings.notify_upcoming => {
                        (AlertKind::Upcoming, "Transferência próxima")
                    }
                    (None, EntryKind::Outcome) if settings.notify_upcoming => {
                        (AlertKind::Upcoming, "Despesa próxima")
                    }
                    (None, EntryKind::Income) if settings.notify_upcoming => {
                        (AlertKind::Upcoming, "Rendimento próximo")
                    }
                    _ => return None,
                };
                Some(Alert {
                    key: format!("occurrence:{}:{}", o.recurrence_id, o.occurrence_date),
                    kind,
                    title: title.into(),
                    message: o.description,
                    date: Some(o.date),
                })
            })
            .collect())
    }

    async fn ending(&self, today: NaiveDate, settings: &Settings) -> Result<Vec<Alert>, AppError> {
        let until = today
            .checked_add_days(Days::new(settings.notify_days_ahead.into()))
            .unwrap_or(today);
        let recurrences = self.recurrences.list().await?;

        Ok(recurrences
            .into_iter()
            .filter(|r| r.is_active())
            .filter_map(|r| {
                let end = r.end_date.filter(|end| *end >= today && *end <= until)?;
                Some(Alert {
                    key: format!("ending:{}:{end}", r.id),
                    kind: AlertKind::Upcoming,
                    title: "Agendamento a terminar".into(),
                    message: format!("{} termina — agendamento a renovar", r.description),
                    date: Some(end),
                })
            })
            .collect())
    }

    async fn goal_alerts(&self, today: NaiveDate) -> Result<Vec<Alert>, AppError> {
        let goals = self.goals.list(today, false).await?;
        Ok(goals
            .into_iter()
            .filter_map(|goal| {
                if goal.current_amount >= goal.target_amount {
                    return Some(Alert {
                        key: format!("goal-reached:{}", goal.id),
                        kind: AlertKind::Goal,
                        title: "Objetivo atingido".into(),
                        message: goal.name,
                        date: None,
                    });
                }
                let target = goal
                    .target_date
                    .filter(|d| (*d - today).num_days() <= GOAL_WARNING_DAYS)?;
                Some(Alert {
                    key: format!("goal-due:{}:{target}", goal.id),
                    kind: AlertKind::Goal,
                    title: "Objetivo perto do prazo".into(),
                    message: goal.name,
                    date: Some(target),
                })
            })
            .collect())
    }

    async fn low_balance(&self, today: NaiveDate, threshold: i64) -> Result<Vec<Alert>, AppError> {
        let accounts = self.accounts.list(today, false).await?;
        Ok(accounts
            .into_iter()
            .filter(|a| a.kind != AccountKind::Card && a.balance < threshold)
            .map(|a| Alert {
                key: format!("low-balance:{}:{today}", a.id),
                kind: AlertKind::LowBalance,
                title: "Saldo baixo".into(),
                message: a.name,
                date: None,
            })
            .collect())
    }

    async fn negative_forecast(
        &self,
        today: NaiveDate,
        settings: &Settings,
    ) -> Result<Vec<Alert>, AppError> {
        let request = ForecastRequest {
            months: FORECAST_MONTHS,
            method: settings.forecast_method,
            history_months: settings.forecast_history_months,
            adjustments: Vec::new(),
            recurrence_changes: Vec::new(),
        };
        let forecast = self.forecasts.forecast(today, &request).await?;
        let accounts = self.accounts.list(today, false).await?;

        Ok(accounts
            .into_iter()
            .filter(|account| account.kind != AccountKind::Card)
            .filter_map(|account| {
                let month = forecast.months.iter().find(|m| {
                    m.balances
                        .iter()
                        .any(|b| b.account_id == account.id && b.balance < 0)
                })?;
                Some(Alert {
                    key: format!("negative:{}:{}", account.id, month.month),
                    kind: AlertKind::NegativeForecast,
                    title: "Previsão de saldo negativo".into(),
                    message: account.name,
                    date: Some(month.month),
                })
            })
            .collect())
    }
}
