use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::AccountKind")]
pub enum AccountKind {
    Bank,
    Savings,
    Card,
    Cash,
    Other,
}

#[derive(SimpleObject)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    pub initial_balance: i64,
    pub color: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct AccountInput {
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    #[graphql(default)]
    pub initial_balance: i64,
    pub color: Option<String>,
}
