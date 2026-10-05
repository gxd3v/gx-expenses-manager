use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WishlistItem {
    pub id: Uuid,
    pub name: String,
    pub amount: i64,
    pub account_id: Uuid,
    pub account_name: String,
    pub priority: u8,
    pub notes: Option<String>,
    pub purchased_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WishlistInput {
    pub name: String,
    pub amount: i64,
    pub account_id: Uuid,
    pub priority: u8,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlanMonth {
    pub month: NaiveDate,
    pub lowest: i64,
    pub lowest_after: i64,
}

#[derive(Debug, Clone)]
pub struct PurchasePlan {
    pub balance: i64,
    pub floor: i64,
    pub lowest_if_today: i64,
    pub earliest: Option<NaiveDate>,
    pub months: Vec<PlanMonth>,
}
