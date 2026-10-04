use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct Goal {
    pub id: Uuid,
    pub name: String,
    pub account_id: Uuid,
    pub account_name: String,
    pub target_amount: i64,
    pub target_date: Option<NaiveDate>,
    pub archived_at: Option<DateTime<Utc>>,
    pub current_amount: i64,
    pub remaining: i64,
    pub progress: f64,
    pub monthly_needed: Option<i64>,
    pub projected_date: Option<NaiveDate>,
}

#[derive(InputObject)]
pub struct GoalInput {
    pub name: String,
    pub account_id: Uuid,
    pub target_amount: i64,
    pub target_date: Option<NaiveDate>,
}
