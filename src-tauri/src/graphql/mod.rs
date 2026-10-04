pub mod accounts;
pub mod backups;
pub mod categories;
pub mod credits;
pub mod forecasts;
pub mod goals;
pub mod reconciliation;
pub mod recurrences;
pub mod reports;
pub mod settings;
pub mod templates;
pub mod transactions;

use async_graphql::{Context, EmptySubscription, MergedObject, Schema};
use sqlx::SqlitePool;

use crate::module::{AppContext, Module};

const MAX_DEPTH: usize = 16;
const MAX_COMPLEXITY: usize = 1000;

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct Query(
    accounts::AccountsQuery,
    categories::CategoriesQuery,
    transactions::TransactionsQuery,
    recurrences::RecurrencesQuery,
    credits::CreditsQuery,
    goals::GoalsQuery,
    forecasts::ForecastsQuery,
    reports::ReportsQuery,
    reconciliation::ReconciliationQuery,
    templates::TemplatesQuery,
    settings::SettingsQuery,
    backups::BackupsQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    accounts::AccountsMutation,
    categories::CategoriesMutation,
    transactions::TransactionsMutation,
    recurrences::RecurrencesMutation,
    credits::CreditsMutation,
    goals::GoalsMutation,
    reconciliation::ReconciliationMutation,
    templates::TemplatesMutation,
    reports::ReportsMutation,
    settings::SettingsMutation,
    backups::BackupsMutation,
);

pub fn schema(pool: SqlitePool, context: AppContext) -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(Module::new(pool, context))
        .limit_depth(MAX_DEPTH)
        .limit_complexity(MAX_COMPLEXITY)
        .finish()
}

fn module<'a>(ctx: &Context<'a>) -> &'a Module {
    ctx.data_unchecked::<Module>()
}
