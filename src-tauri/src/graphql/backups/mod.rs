pub mod types;

use std::path::PathBuf;

use async_graphql::{Context, Object, Result};

use super::module;
use crate::graphql::transactions::types::TransactionFilter;
use types::{BackupFile, ImportPreview, ImportResult};

#[derive(Default)]
pub struct BackupsQuery;

#[Object]
impl BackupsQuery {
    async fn backups(&self, ctx: &Context<'_>) -> Result<Vec<BackupFile>> {
        let backups = module(ctx).backups.list().await?;
        Ok(backups.into_iter().map(BackupFile::from).collect())
    }

    async fn inspect_import(
        &self,
        ctx: &Context<'_>,
        path: String,
        password: Option<String>,
    ) -> Result<ImportPreview> {
        Ok(module(ctx)
            .backups
            .inspect(&PathBuf::from(path), password)
            .await?
            .into())
    }
}

#[derive(Default)]
pub struct BackupsMutation;

#[Object]
impl BackupsMutation {
    async fn create_backup(&self, ctx: &Context<'_>) -> Result<BackupFile> {
        Ok(module(ctx).backups.create_backup("").await?.into())
    }

    async fn auto_backup(&self, ctx: &Context<'_>) -> Result<Option<BackupFile>> {
        Ok(module(ctx).backups.auto_backup().await?.map(Into::into))
    }

    async fn verify_backup(
        &self,
        ctx: &Context<'_>,
        path: String,
        password: Option<String>,
    ) -> Result<bool> {
        module(ctx)
            .backups
            .verify(&PathBuf::from(path), password)
            .await?;
        Ok(true)
    }

    async fn export_data(
        &self,
        ctx: &Context<'_>,
        path: String,
        password: Option<String>,
    ) -> Result<bool> {
        module(ctx)
            .backups
            .export(&PathBuf::from(path), password)
            .await?;
        Ok(true)
    }

    async fn export_csv(
        &self,
        ctx: &Context<'_>,
        path: String,
        #[graphql(default)] filter: TransactionFilter,
    ) -> Result<usize> {
        Ok(module(ctx)
            .backups
            .export_csv(&PathBuf::from(path), &filter.into())
            .await?)
    }

    async fn import_data(
        &self,
        ctx: &Context<'_>,
        path: String,
        password: Option<String>,
        replace: bool,
    ) -> Result<ImportResult> {
        Ok(module(ctx)
            .backups
            .import(&PathBuf::from(path), password, replace)
            .await?
            .into())
    }
}
