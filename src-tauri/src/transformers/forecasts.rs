use crate::graphql::forecasts::types;
use crate::managers::forecasts::{Forecast, ForecastRequest, Milestone, RecurrenceChange};
use crate::models;

impl From<Forecast> for types::Forecast {
    fn from(forecast: Forecast) -> Self {
        let total_income: i64 = forecast.months.iter().map(|m| m.income).sum();
        let total_outcome: i64 = forecast.months.iter().map(|m| m.outcome).sum();
        Self {
            total_income,
            total_outcome,
            total_net: total_income - total_outcome,
            months: forecast.months.into_iter().map(Into::into).collect(),
            goals_reached: forecast.goals_reached.into_iter().map(Into::into).collect(),
            credits_paid: forecast.credits_paid.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<models::ForecastMonth> for types::ForecastMonth {
    fn from(month: models::ForecastMonth) -> Self {
        Self {
            month: month.month,
            income: month.income,
            outcome: month.outcome,
            net: month.income - month.outcome,
            fixed_income: month.income - month.variable_income,
            fixed_outcome: month.outcome - month.variable_outcome,
            variable_income: month.variable_income,
            variable_outcome: month.variable_outcome,
            interest: month.interest,
            total: month.total,
            balances: month
                .balances
                .into_iter()
                .map(|b| types::AccountBalance {
                    account_id: b.account_id,
                    balance: b.balance,
                })
                .collect(),
        }
    }
}

impl From<Milestone> for types::Milestone {
    fn from(milestone: Milestone) -> Self {
        Self {
            id: milestone.id,
            name: milestone.name,
            date: milestone.date,
        }
    }
}

impl types::ForecastInput {
    pub fn into_request(self, settings: &models::Settings) -> ForecastRequest {
        ForecastRequest {
            months: self.months,
            method: self.method.map_or(settings.forecast_method, Into::into),
            history_months: self
                .history_months
                .unwrap_or(settings.forecast_history_months),
            adjustments: self
                .adjustments
                .into_iter()
                .map(|a| models::Adjustment {
                    account_id: a.account_id,
                    to_account_id: a.to_account_id,
                    amount: a.amount,
                    date: a.date,
                    repeat_months: a.repeat_months,
                })
                .collect(),
            recurrence_changes: self
                .recurrence_changes
                .into_iter()
                .map(|c| RecurrenceChange {
                    recurrence_id: c.recurrence_id,
                    amount: c.amount,
                })
                .collect(),
        }
    }
}
