pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use crate::models::dates::today;
use types::{Account, AccountInput};

#[derive(Default)]
pub struct AccountsQuery;

#[Object]
impl AccountsQuery {
    async fn accounts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] include_archived: bool,
    ) -> Result<Vec<Account>> {
        let accounts = module(ctx).accounts.list(today(), include_archived).await?;
        Ok(accounts.into_iter().map(Account::from).collect())
    }

    async fn account(&self, ctx: &Context<'_>, id: Uuid) -> Result<Account> {
        Ok(module(ctx).accounts.get(today(), id).await?.into())
    }
}

#[derive(Default)]
pub struct AccountsMutation;

#[Object]
impl AccountsMutation {
    async fn create_account(&self, ctx: &Context<'_>, input: AccountInput) -> Result<Account> {
        Ok(module(ctx)
            .accounts
            .create(today(), input.into())
            .await?
            .into())
    }

    async fn update_account(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: AccountInput,
    ) -> Result<Account> {
        Ok(module(ctx)
            .accounts
            .update(today(), id, input.into())
            .await?
            .into())
    }

    async fn set_account_archived(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        archived: bool,
    ) -> Result<Account> {
        Ok(module(ctx)
            .accounts
            .set_archived(today(), id, archived)
            .await?
            .into())
    }

    async fn reorder_accounts(&self, ctx: &Context<'_>, ids: Vec<Uuid>) -> Result<bool> {
        module(ctx).accounts.reorder(&ids).await?;
        Ok(true)
    }

    async fn delete_account(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).accounts.delete(id).await?;
        Ok(true)
    }
}
