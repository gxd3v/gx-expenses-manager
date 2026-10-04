pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use crate::models::dates::today;
use types::{Credit, CreditInput, CreditPayment, PaymentInput, Simulation, SimulationInput};

#[derive(Default)]
pub struct CreditsQuery;

#[Object]
impl CreditsQuery {
    async fn credits(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] include_archived: bool,
    ) -> Result<Vec<Credit>> {
        let today = today();
        let credits = module(ctx).credits.list(include_archived).await?;
        Ok(credits
            .into_iter()
            .map(|c| Credit::from_model(c, today))
            .collect())
    }

    async fn credit(&self, ctx: &Context<'_>, id: Uuid) -> Result<Credit> {
        Ok(Credit::from_model(
            module(ctx).credits.get(id).await?,
            today(),
        ))
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
        let today = today();
        let credit = module(ctx)
            .credits
            .create(today, input.try_into()?, create_recurrence)
            .await?;
        Ok(Credit::from_model(credit, today))
    }

    async fn update_credit(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: CreditInput,
    ) -> Result<Credit> {
        Ok(Credit::from_model(
            module(ctx).credits.update(id, input.try_into()?).await?,
            today(),
        ))
    }

    async fn set_credit_archived(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        archived: bool,
    ) -> Result<Credit> {
        Ok(Credit::from_model(
            module(ctx).credits.set_archived(id, archived).await?,
            today(),
        ))
    }

    async fn delete_credit(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).credits.delete(id).await?;
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
