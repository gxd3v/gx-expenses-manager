pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use types::{Category, CategoryInput};

#[derive(Default)]
pub struct CategoriesQuery;

#[Object]
impl CategoriesQuery {
    async fn categories(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] include_archived: bool,
    ) -> Result<Vec<Category>> {
        let categories = module(ctx).categories.list(include_archived).await?;
        Ok(categories.into_iter().map(Category::from).collect())
    }
}

#[derive(Default)]
pub struct CategoriesMutation;

#[Object]
impl CategoriesMutation {
    async fn create_category(&self, ctx: &Context<'_>, input: CategoryInput) -> Result<Category> {
        Ok(module(ctx).categories.create(input.into()).await?.into())
    }

    async fn update_category(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: CategoryInput,
    ) -> Result<Category> {
        Ok(module(ctx)
            .categories
            .update(id, input.into())
            .await?
            .into())
    }

    async fn set_category_archived(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        archived: bool,
    ) -> Result<Category> {
        Ok(module(ctx)
            .categories
            .set_archived(id, archived)
            .await?
            .into())
    }

    async fn delete_category(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        reassign_to: Option<Uuid>,
    ) -> Result<bool> {
        module(ctx).categories.delete(id, reassign_to).await?;
        Ok(true)
    }
}
