pub mod types;

use async_graphql::{Context, Object, Result};

use super::module;
use crate::managers::forecasts::ForecastRequest;
use crate::models::dates::today;
use types::{Forecast, ForecastInput, ScenarioComparison};

#[derive(Default)]
pub struct ForecastsQuery;

#[Object]
impl ForecastsQuery {
    async fn forecast(&self, ctx: &Context<'_>, input: ForecastInput) -> Result<Forecast> {
        let request = request(ctx, input).await?;
        Ok(module(ctx)
            .forecasts
            .forecast(today(), &request)
            .await?
            .into())
    }

    async fn forecast_scenario(
        &self,
        ctx: &Context<'_>,
        input: ForecastInput,
    ) -> Result<ScenarioComparison> {
        let scenario = request(ctx, input).await?;
        let base = ForecastRequest {
            adjustments: Vec::new(),
            recurrence_changes: Vec::new(),
            ..scenario.clone()
        };

        let forecasts = &module(ctx).forecasts;
        let base: Forecast = forecasts.forecast(today(), &base).await?.into();
        let scenario: Forecast = forecasts.forecast(today(), &scenario).await?.into();
        let end = |forecast: &Forecast| forecast.months.last().map_or(0, |m| m.total);

        Ok(ScenarioComparison {
            end_difference: end(&scenario) - end(&base),
            base,
            scenario,
        })
    }
}

async fn request(ctx: &Context<'_>, input: ForecastInput) -> Result<ForecastRequest> {
    let settings = module(ctx).settings.get().await?;
    Ok(input.into_request(&settings))
}
