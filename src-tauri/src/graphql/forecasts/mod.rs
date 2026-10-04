pub mod types;

use async_graphql::{Context, Object, Result};

use super::module;
use crate::managers::forecasts::{ForecastRequest, RecurrenceChange};
use crate::models::Adjustment;
use crate::models::dates::today;
use types::{Forecast, ForecastInput};

#[derive(Default)]
pub struct ForecastsQuery;

#[Object]
impl ForecastsQuery {
    async fn forecast(&self, ctx: &Context<'_>, input: ForecastInput) -> Result<Forecast> {
        let module = module(ctx);
        let settings = module.settings.get().await?;

        let request = ForecastRequest {
            months: input.months,
            method: input.method.map_or(settings.forecast_method, Into::into),
            history_months: input
                .history_months
                .unwrap_or(settings.forecast_history_months),
            adjustments: input
                .adjustments
                .into_iter()
                .map(|a| Adjustment {
                    account_id: a.account_id,
                    to_account_id: a.to_account_id,
                    amount: a.amount,
                    date: a.date,
                    repeat_months: a.repeat_months,
                })
                .collect(),
            recurrence_changes: input
                .recurrence_changes
                .into_iter()
                .map(|c| RecurrenceChange {
                    recurrence_id: c.recurrence_id,
                    amount: c.amount,
                })
                .collect(),
        };

        Ok(module.forecasts.forecast(today(), &request).await?.into())
    }
}
