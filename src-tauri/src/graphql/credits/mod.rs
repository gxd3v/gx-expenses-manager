pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use crate::models;
use crate::models::dates::today;
use types::{
    Credit, CreditBalance, CreditInput, CreditPayment, PaymentInput, Simulation, SimulationInput,
};

#[derive(Default)]
pub struct CreditsQuery;

#[Object]
impl CreditsQuery {
    async fn credits(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] include_archived: bool,
    ) -> Result<Vec<Credit>> {
        let manager = &module(ctx).credits;
        let links = manager.recurrence_links().await?;
        let credits = manager.list(include_archived).await?;
        Ok(credits
            .into_iter()
            .map(|c| {
                let recurrence_id = links.get(&c.id).copied();
                Credit::from_model(c, today(), recurrence_id)
            })
            .collect())
    }

    async fn credit(&self, ctx: &Context<'_>, id: Uuid) -> Result<Credit> {
        let credit = module(ctx).credits.get(id).await?;
        to_graphql(ctx, credit).await
    }

    async fn credit_history(&self, ctx: &Context<'_>, id: Uuid) -> Result<Vec<CreditBalance>> {
        let history = module(ctx).credits.balance_history(id, today()).await?;
        Ok(history.into_iter().map(CreditBalance::from).collect())
    }

    async fn credit_payments(
        &self,
        ctx: &Context<'_>,
        credit_id: Option<Uuid>,
    ) -> Result<Vec<CreditPayment>> {
        let payments = module(ctx).credits.payments(credit_id).await?;
        Ok(payments.into_iter().map(CreditPayment::from).collect())
    }

    async fn simulate_credit(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: SimulationInput,
    ) -> Result<Simulation> {
        Ok(module(ctx)
            .credits
            .simulate(today(), id, input.into())
            .await?
            .into())
    }
}

#[derive(Default)]
pub struct CreditsMutation;

#[Object]
impl CreditsMutation {
    async fn create_credit(
        &self,
        ctx: &Context<'_>,
        input: CreditInput,
        #[graphql(default)] create_recurrence: bool,
    ) -> Result<Credit> {
        let credit = module(ctx)
            .credits
            .create(today(), input.try_into()?, create_recurrence)
            .await?;
        to_graphql(ctx, credit).await
    }

    async fn update_credit(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: CreditInput,
    ) -> Result<Credit> {
        let credit = module(ctx).credits.update(id, input.try_into()?).await?;
        to_graphql(ctx, credit).await
    }

    async fn set_credit_archived(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        archived: bool,
    ) -> Result<Credit> {
        let credit = module(ctx).credits.set_archived(id, archived).await?;
        to_graphql(ctx, credit).await
    }

    async fn delete_credit(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).credits.delete(id).await?;
        Ok(true)
    }

    async fn create_credit_recurrence(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).credits.add_recurrence(id, today()).await?;
        Ok(true)
    }

    async fn register_credit_payment(
        &self,
        ctx: &Context<'_>,
        input: PaymentInput,
    ) -> Result<CreditPayment> {
        Ok(module(ctx)
            .credits
            .register_payment(input.into())
            .await?
            .into())
    }

    async fn delete_credit_payment(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).credits.delete_payment(id).await?;
        Ok(true)
    }
}

async fn to_graphql(ctx: &Context<'_>, credit: models::Credit) -> Result<Credit> {
    let links = module(ctx).credits.recurrence_links().await?;
    let recurrence_id = links.get(&credit.id).copied();
    Ok(Credit::from_model(credit, today(), recurrence_id))
}
