use chrono::{DateTime, Utc};
use uuid::Uuid;

string_enum!(AccountKind {
    Bank => "bank",
    Savings => "savings",
    Card => "card",
    Cash => "cash",
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
}

#[derive(Debug, Clone)]
pub struct AccountInput {
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    pub initial_balance: i64,
    pub color: Option<String>,
    pub icon: Option<String>,
}
