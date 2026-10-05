use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(InputObject)]
pub struct PurchasePlanInput {
    pub account_id: Uuid,
    pub amount: i64,
    #[graphql(default)]
    pub margin: i64,
    #[graphql(default)]
    pub allow_overdraft: bool,
    #[graphql(default = 12)]
    pub months: u32,
}

#[derive(SimpleObject)]
pub struct PlanMonth {
    pub month: NaiveDate,
    pub lowest: i64,
    pub lowest_after: i64,
}

#[derive(SimpleObject)]
pub struct PurchasePlan {
    pub balance: i64,
    pub floor: i64,
    pub lowest_if_today: i64,
    pub earliest_date: Option<NaiveDate>,
    pub months: Vec<PlanMonth>,
}

#[derive(SimpleObject)]
pub struct WishlistItem {
    pub id: Uuid,
    pub name: String,
    pub amount: i64,
    pub account_id: Uuid,
    pub account_name: String,
    pub priority: u8,
    pub notes: Option<String>,
    pub purchased_at: Option<DateTime<Utc>>,
    pub planned_date: Option<NaiveDate>,
}

#[derive(InputObject)]
pub struct WishlistInput {
    pub name: String,
    pub amount: i64,
    pub account_id: Uuid,
    #[graphql(default = 2)]
    pub priority: u8,
    pub notes: Option<String>,
}
