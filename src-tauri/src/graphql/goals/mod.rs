pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use crate::models::dates::today;
use types::{Goal, GoalInput};

#[derive(Default)]
pub struct GoalsQuery;

#[Object]
impl GoalsQuery {
    async fn goals(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] include_archived: bool,
    ) -> Result<Vec<Goal>> {
        let today = today();
        let manager = &module(ctx).goals;
        let projections = manager.projections(today).await?;
        let goals = manager.list(today, include_archived).await?;
        Ok(goals
            .into_iter()
            .map(|g| {
                let projected = projections.get(&g.id).copied();
                Goal::from_model(g, today, projected)
            })
            .collect())
    }
}

#[derive(Default)]
pub struct GoalsMutation;

#[Object]
impl GoalsMutation {
    async fn create_goal(&self, ctx: &Context<'_>, input: GoalInput) -> Result<Goal> {
        let today = today();
        Ok(Goal::from_model(
            module(ctx).goals.create(today, input.into()).await?,
            today,
            None,
        ))
    }

    async fn update_goal(&self, ctx: &Context<'_>, id: Uuid, input: GoalInput) -> Result<Goal> {
        let today = today();
        Ok(Goal::from_model(
            module(ctx).goals.update(today, id, input.into()).await?,
            today,
            None,
        ))
    }

    async fn set_goal_archived(&self, ctx: &Context<'_>, id: Uuid, archived: bool) -> Result<Goal> {
        let today = today();
        Ok(Goal::from_model(
            module(ctx).goals.set_archived(today, id, archived).await?,
            today,
            None,
        ))
    }

    async fn delete_goal(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).goals.delete(id).await?;
        Ok(true)
    }
}
