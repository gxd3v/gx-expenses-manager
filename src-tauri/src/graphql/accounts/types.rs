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
    CreditCard,
    Cash,
    Meal,
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
    pub icon: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub balance: i64,
    pub available_balance: i64,
    pub projected_balance: i64,
    pub interest: Option<Interest>,
    pub interest_rate: Option<f64>,
    pub estimated_interest: Option<i64>,
    pub overdraft_limit: i64,
}

#[derive(SimpleObject, InputObject)]
#[graphql(input_name = "InterestTierInput")]
pub struct InterestTier {
    pub min_balance: i64,
    pub rate: f64,
}

#[derive(SimpleObject, InputObject)]
#[graphql(input_name = "InterestInput")]
pub struct Interest {
    pub period_months: u32,
    pub tiers: Vec<InterestTier>,
}

#[derive(InputObject)]
pub struct AccountInput {
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    #[graphql(default)]
    pub initial_balance: i64,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub interest: Option<Interest>,
    #[graphql(default)]
    pub overdraft_limit: i64,
}
