pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use crate::models::dates::today;
use types::{PurchasePlan, PurchasePlanInput, WishlistInput, WishlistItem};

#[derive(Default)]
pub struct PurchasesQuery;

#[Object]
impl PurchasesQuery {
    async fn purchase_plan(
        &self,
        ctx: &Context<'_>,
        input: PurchasePlanInput,
    ) -> Result<PurchasePlan> {
        let plan = module(ctx).purchases.plan(today(), &input.into()).await?;
        Ok(plan.into())
    }

    async fn wishlist(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] margin: i64,
        #[graphql(default)] allow_overdraft: bool,
    ) -> Result<Vec<WishlistItem>> {
        let items = module(ctx)
            .purchases
            .wishlist(today(), margin, allow_overdraft)
            .await?;
        Ok(items
            .into_iter()
            .map(|(item, date)| WishlistItem::from_model(item, date))
            .collect())
    }
}

#[derive(Default)]
pub struct PurchasesMutation;

#[Object]
impl PurchasesMutation {
    async fn create_wishlist_item(
        &self,
        ctx: &Context<'_>,
        input: WishlistInput,
    ) -> Result<WishlistItem> {
        let item = module(ctx).purchases.create(input.into()).await?;
        Ok(WishlistItem::from_model(item, None))
    }

    async fn update_wishlist_item(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: WishlistInput,
    ) -> Result<WishlistItem> {
        let item = module(ctx).purchases.update(id, input.into()).await?;
        Ok(WishlistItem::from_model(item, None))
    }

    async fn set_wishlist_item_purchased(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        purchased: bool,
    ) -> Result<WishlistItem> {
        let item = module(ctx).purchases.set_purchased(id, purchased).await?;
        Ok(WishlistItem::from_model(item, None))
    }

    async fn delete_wishlist_item(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).purchases.delete(id).await?;
        Ok(true)
    }
}
