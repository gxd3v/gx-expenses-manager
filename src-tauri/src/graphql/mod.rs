pub mod accounts;

use async_graphql::{Context, EmptySubscription, MergedObject, Schema};
use sqlx::SqlitePool;

use crate::module::Module;
use accounts::{AccountsMutation, AccountsQuery};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct Query(AccountsQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(AccountsMutation);

pub fn schema(pool: SqlitePool) -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(Module::new(pool))
        .finish()
}

fn module<'a>(ctx: &Context<'a>) -> &'a Module {
    ctx.data_unchecked::<Module>()
}
