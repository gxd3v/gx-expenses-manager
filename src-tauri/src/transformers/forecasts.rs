use crate::graphql::forecasts::types;
use crate::managers::forecasts::{Forecast, Milestone};
use crate::models;

impl From<Forecast> for types::Forecast {
    fn from(forecast: Forecast) -> Self {
        Self {
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
