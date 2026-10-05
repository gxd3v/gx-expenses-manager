use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use super::forecasts::{ForecastRequest, ForecastsManager};
use super::{archived_at, positive, required};
use crate::errors::AppError;
use crate::models::{ForecastMethod, Goal, GoalInput};
use crate::repositories::goals::GoalsRepository;

const PROJECTION_MONTHS: u32 = 120;

#[derive(Clone)]
pub struct GoalsManager {
    repository: GoalsRepository,
    forecasts: ForecastsManager,
}

impl GoalsManager {
    pub fn new(repository: GoalsRepository, forecasts: ForecastsManager) -> Self {
        Self {
            repository,
            forecasts,
        }
    }

    pub async fn list(
        &self,
        today: NaiveDate,
        include_archived: bool,
    ) -> Result<Vec<Goal>, AppError> {
        self.repository.list(today, include_archived).await
    }

    pub async fn projections(
        &self,
        today: NaiveDate,
    ) -> Result<HashMap<Uuid, NaiveDate>, AppError> {
        let request = ForecastRequest {
            months: PROJECTION_MONTHS,
            method: ForecastMethod::Recurring,
            history_months: 6,
            adjustments: Vec::new(),
            recurrence_changes: Vec::new(),
        };
        let forecast = self.forecasts.forecast(today, &request).await?;
        Ok(forecast
            .goals_reached
            .into_iter()
            .map(|m| (m.id, m.date))
            .collect())
    }

    pub async fn create(&self, today: NaiveDate, input: GoalInput) -> Result<Goal, AppError> {
        let input = normalize(input)?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, Utc::now()).await?;
        self.repository.get(today, id).await
    }

    pub async fn update(
        &self,
        today: NaiveDate,
        id: Uuid,
        input: GoalInput,
    ) -> Result<Goal, AppError> {
        let input = normalize(input)?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(today, id).await
    }

    pub async fn set_archived(
        &self,
        today: NaiveDate,
        id: Uuid,
        archived: bool,
    ) -> Result<Goal, AppError> {
        self.repository
            .set_archived(id, archived_at(archived), Utc::now())
            .await?;
        self.repository.get(today, id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

fn normalize(input: GoalInput) -> Result<GoalInput, AppError> {
    positive(
        input.target_amount,
        "o valor objetivo tem de ser maior que zero",
    )?;
    Ok(GoalInput {
        name: required(&input.name, "o nome é obrigatório")?,
        ..input
    })
}
