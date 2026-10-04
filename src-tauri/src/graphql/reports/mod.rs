pub mod types;

use async_graphql::{Context, Object, Result};
use chrono::NaiveDate;
use uuid::Uuid;

use super::module;
use crate::graphql::transactions::types::EntryKind;
use crate::models::dates::today;
use types::{
    Alert, BalancePoint, BalanceRecord, BalanceSummary, CategoryAmount, CategoryGrouping,
    MonthComparison, MonthSummary, MonthlyTotal,
};

#[derive(Default)]
pub struct ReportsQuery;

#[Object]
impl ReportsQuery {
    async fn monthly_totals(
        &self,
        ctx: &Context<'_>,
        from: NaiveDate,
        to: NaiveDate,
        account_id: Option<Uuid>,
    ) -> Result<Vec<MonthlyTotal>> {
        let totals = module(ctx)
            .reports
            .monthly_totals(from, to, account_id)
            .await?;
        Ok(totals.into_iter().map(MonthlyTotal::from).collect())
    }

    async fn category_breakdown(
        &self,
        ctx: &Context<'_>,
        kind: EntryKind,
        from: NaiveDate,
        to: NaiveDate,
        #[graphql(default_with = "CategoryGrouping::Parent")] grouping: CategoryGrouping,
        account_id: Option<Uuid>,
    ) -> Result<Vec<CategoryAmount>> {
        let amounts = module(ctx)
            .reports
            .by_category(kind.into(), from, to, grouping.into(), account_id)
            .await?;
        Ok(amounts.into_iter().map(CategoryAmount::from).collect())
    }

    async fn balance_history(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 12)] months: u32,
        account_id: Option<Uuid>,
    ) -> Result<Vec<BalancePoint>> {
        let points = module(ctx)
            .reports
            .balance_history(today(), months, account_id)
            .await?;
        Ok(points.into_iter().map(BalancePoint::from).collect())
    }

    async fn balance_records(
        &self,
        ctx: &Context<'_>,
        account_id: Option<Uuid>,
    ) -> Result<Vec<BalanceRecord>> {
        let module = module(ctx);
        let settings = module.settings.get().await?;
        let records = module
            .reports
            .balance_records(today(), settings.first_day_of_week, account_id)
            .await?;
        Ok(records.into_iter().map(BalanceRecord::from).collect())
    }

    async fn month_summary(&self, ctx: &Context<'_>, month: NaiveDate) -> Result<MonthSummary> {
        Ok(module(ctx)
            .reports
            .month_summary(today(), month)
            .await?
            .into())
    }

    async fn balance_summary(&self, ctx: &Context<'_>) -> Result<BalanceSummary> {
        Ok(module(ctx).reports.balance_summary(today()).await?.into())
    }

    async fn category_averages(
        &self,
        ctx: &Context<'_>,
        kind: EntryKind,
        #[graphql(default = 6)] months: u32,
    ) -> Result<Vec<CategoryAmount>> {
        let averages = module(ctx)
            .reports
            .category_averages(today(), kind.into(), months)
            .await?;
        Ok(averages.into_iter().map(CategoryAmount::from).collect())
    }

    async fn compare_months(
        &self,
        ctx: &Context<'_>,
        first: NaiveDate,
        second: NaiveDate,
    ) -> Result<Vec<MonthComparison>> {
        let rows = module(ctx).reports.compare_months(first, second).await?;
        Ok(rows.into_iter().map(MonthComparison::from).collect())
    }

    async fn alerts(&self, ctx: &Context<'_>) -> Result<Vec<Alert>> {
        let alerts = module(ctx).alerts.alerts(today()).await?;
        Ok(alerts.into_iter().map(Alert::from).collect())
    }
}
