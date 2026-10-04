pub mod types;

use async_graphql::{Context, Object, Result};

use super::module;
use types::Settings;

#[derive(Default)]
pub struct SettingsQuery;

#[Object]
impl SettingsQuery {
    async fn settings(&self, ctx: &Context<'_>) -> Result<Settings> {
        Ok(module(ctx).settings.get().await?.into())
    }
}

#[derive(Default)]
pub struct SettingsMutation;

#[Object]
impl SettingsMutation {
    async fn update_settings(&self, ctx: &Context<'_>, input: Settings) -> Result<Settings> {
        Ok(module(ctx).settings.update(input.into()).await?.into())
    }
}
