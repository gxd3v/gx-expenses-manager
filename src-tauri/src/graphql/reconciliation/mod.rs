pub mod types;

use async_graphql::{Context, Object, Result};
use chrono::NaiveDate;
use uuid::Uuid;

use super::module;
use types::{ForgottenCandidate, Reconciliation, ReconciliationStatus};

#[derive(Default)]
pub struct ReconciliationQuery;

#[Object]
impl ReconciliationQuery {
    async fn reconciliation_status(
        &self,
        ctx: &Context<'_>,
        account_id: Uuid,
        date: NaiveDate,
        statement_balance: Option<i64>,
    ) -> Result<ReconciliationStatus> {
        let status = module(ctx).reconciliation.status(account_id, date).await?;
        Ok(ReconciliationStatus::from_model(status, statement_balance))
    }

    async fn reconciliations(
        &self,
        ctx: &Context<'_>,
        account_id: Uuid,
    ) -> Result<Vec<Reconciliation>> {
        let history = module(ctx).reconciliation.history(account_id).await?;
        Ok(history.into_iter().map(Reconciliation::from).collect())
    }

    async fn forgotten_transactions(
        &self,
        ctx: &Context<'_>,
        account_id: Uuid,
        month: NaiveDate,
    ) -> Result<Vec<ForgottenCandidate>> {
        let candidates = module(ctx)
            .reconciliation
            .forgotten(account_id, month)
            .await?;
        Ok(candidates
            .into_iter()
            .map(ForgottenCandidate::from)
            .collect())
    }
}

#[derive(Default)]
pub struct ReconciliationMutation;

#[Object]
impl ReconciliationMutation {
    async fn reconcile(
        &self,
        ctx: &Context<'_>,
        account_id: Uuid,
        date: NaiveDate,
        statement_balance: i64,
        confirm_from: Option<NaiveDate>,
    ) -> Result<Reconciliation> {
        let reconciliation = module(ctx)
            .reconciliation
            .reconcile(account_id, date, statement_balance, confirm_from)
            .await?;
        Ok(reconciliation.into())
    }
}
