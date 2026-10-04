pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use crate::graphql::credits::types::Credit;
use crate::graphql::goals::types::Goal;
use crate::models::dates::today;
use types::{
    SavedFilter, SearchResult, Transaction, TransactionFilter, TransactionInput, TransactionPage,
    Transfer, TransferInput,
};

#[derive(Default)]
pub struct TransactionsQuery;

#[Object]
impl TransactionsQuery {
    async fn transactions(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] filter: TransactionFilter,
        #[graphql(default = 50)] limit: i64,
        #[graphql(default)] offset: i64,
    ) -> Result<TransactionPage> {
        Ok(module(ctx)
            .transactions
            .page(&filter.into(), limit, offset)
            .await?
            .into())
    }

    async fn pending_confirmations(&self, ctx: &Context<'_>) -> Result<Vec<Transaction>> {
        let items = module(ctx)
            .transactions
            .pending_confirmations(today())
            .await?;
        Ok(items.into_iter().map(Transaction::from).collect())
    }

    async fn transaction(&self, ctx: &Context<'_>, id: Uuid) -> Result<Transaction> {
        Ok(module(ctx).transactions.get(id).await?.into())
    }

    async fn transfer(&self, ctx: &Context<'_>, id: Uuid) -> Result<Transfer> {
        Ok(module(ctx).transfers.get(id).await?.into())
    }

    async fn saved_filters(&self, ctx: &Context<'_>) -> Result<Vec<SavedFilter>> {
        let filters = module(ctx).search.saved_filters().await?;
        Ok(filters.into_iter().map(SavedFilter::from).collect())
    }

    async fn search(&self, ctx: &Context<'_>, text: String) -> Result<SearchResult> {
        let today = today();
        let result = module(ctx).search.search(today, &text).await?;
        Ok(SearchResult {
            accounts: result.accounts.into_iter().map(Into::into).collect(),
            categories: result.categories.into_iter().map(Into::into).collect(),
            transactions: result.transactions.into_iter().map(Into::into).collect(),
            credits: result
                .credits
                .into_iter()
                .map(|c| Credit::from_model(c, today, None))
                .collect(),
            goals: result
                .goals
                .into_iter()
                .map(|g| Goal::from_model(g, today, None))
                .collect(),
        })
    }
}

#[derive(Default)]
pub struct TransactionsMutation;

#[Object]
impl TransactionsMutation {
    async fn create_transaction(
        &self,
        ctx: &Context<'_>,
        input: TransactionInput,
    ) -> Result<Transaction> {
        Ok(module(ctx).transactions.create(input.into()).await?.into())
    }

    async fn update_transaction(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: TransactionInput,
    ) -> Result<Transaction> {
        Ok(module(ctx)
            .transactions
            .update(id, input.into())
            .await?
            .into())
    }

    async fn delete_transaction(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).transactions.delete(id).await?;
        Ok(true)
    }

    async fn set_transactions_confirmed(
        &self,
        ctx: &Context<'_>,
        ids: Vec<Uuid>,
        confirmed: bool,
    ) -> Result<u64> {
        Ok(module(ctx)
            .transactions
            .set_confirmed(&ids, confirmed)
            .await?)
    }

    async fn create_transfer(&self, ctx: &Context<'_>, input: TransferInput) -> Result<Transfer> {
        Ok(module(ctx)
            .transfers
            .create(today(), input.into())
            .await?
            .into())
    }

    async fn update_transfer(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: TransferInput,
    ) -> Result<Transfer> {
        Ok(module(ctx)
            .transfers
            .update(today(), id, input.into())
            .await?
            .into())
    }

    async fn delete_transfer(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).transfers.delete(id).await?;
        Ok(true)
    }

    async fn save_filter(
        &self,
        ctx: &Context<'_>,
        name: String,
        filter: String,
    ) -> Result<SavedFilter> {
        Ok(module(ctx).search.save_filter(&name, filter).await?.into())
    }

    async fn delete_saved_filter(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).search.delete_filter(id).await?;
        Ok(true)
    }
}
