pub mod types;

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use super::module;
use types::{Template, TemplateInput};

#[derive(Default)]
pub struct TemplatesQuery;

#[Object]
impl TemplatesQuery {
    async fn templates(&self, ctx: &Context<'_>) -> Result<Vec<Template>> {
        let templates = module(ctx).templates.list().await?;
        Ok(templates.into_iter().map(Template::from).collect())
    }
}

#[derive(Default)]
pub struct TemplatesMutation;

#[Object]
impl TemplatesMutation {
    async fn create_template(&self, ctx: &Context<'_>, input: TemplateInput) -> Result<Template> {
        Ok(module(ctx).templates.create(input.into()).await?.into())
    }

    async fn update_template(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: TemplateInput,
    ) -> Result<Template> {
        Ok(module(ctx).templates.update(id, input.into()).await?.into())
    }

    async fn delete_template(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).templates.delete(id).await?;
        Ok(true)
    }
}
