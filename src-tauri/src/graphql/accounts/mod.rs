pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use types::{Account, AccountInput};

#[derive(Default)]
pub struct AccountsQuery;

#[Object]
impl AccountsQuery {
    async fn accounts(&self, ctx: &Context<'_>, #[graphql(default)] include_archived: bool) -> Result<Vec<Account>> {
        let accounts = module(ctx).accounts.list(include_archived).await?;
        Ok(accounts.into_iter().map(Account::from).collect())
    }
}

#[derive(Default)]
pub struct AccountsMutation;

#[Object]
impl AccountsMutation {
    async fn create_account(&self, ctx: &Context<'_>, input: AccountInput) -> Result<Account> {
        let account = module(ctx).accounts.create(input.into()).await?;
        Ok(account.into())
    }

    async fn update_account(&self, ctx: &Context<'_>, id: Uuid, input: AccountInput) -> Result<Account> {
        let account = module(ctx).accounts.update(id, input.into()).await?;
        Ok(account.into())
    }

    async fn archive_account(&self, ctx: &Context<'_>, id: Uuid) -> Result<Account> {
        let account = module(ctx).accounts.archive(id).await?;
        Ok(account.into())
    }
}
