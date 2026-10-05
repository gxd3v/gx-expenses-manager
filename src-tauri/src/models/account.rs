use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Interest;

string_enum!(AccountKind {
    Bank => "bank",
    Savings => "savings",
    Card => "card",
    CreditCard => "credit_card",
    Cash => "cash",
    Meal => "meal",
    Other => "other",
});

#[derive(Debug, Clone)]
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
    pub overdraft_limit: i64,
}

impl Account {
    pub fn counts_in_total(&self) -> bool {
        self.kind != AccountKind::CreditCard
    }
}

#[derive(Debug, Clone)]
pub struct AccountInput {
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    pub initial_balance: i64,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub interest: Option<Interest>,
    pub overdraft_limit: i64,
}
