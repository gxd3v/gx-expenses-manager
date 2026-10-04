use async_graphql::{Enum, SimpleObject};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::CategoryGrouping")]
pub enum CategoryGrouping {
    Parent,
    Leaf,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::RecordPeriod")]
pub enum RecordPeriod {
    AllTime,
    Year,
    Month,
    Week,
}

#[derive(SimpleObject)]
pub struct BalanceMark {
    pub date: NaiveDate,
    pub balance: i64,
}

#[derive(SimpleObject)]
pub struct BalanceRecord {
    pub period: RecordPeriod,
    pub high: BalanceMark,
    pub low: BalanceMark,
}

#[derive(SimpleObject)]
pub struct MonthlyTotal {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
    pub net: i64,
}

#[derive(SimpleObject)]
pub struct CategoryAmount {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub amount: i64,
}

#[derive(SimpleObject)]
pub struct BalancePoint {
    pub month: NaiveDate,
    pub balance: i64,
    pub debt: i64,
    pub net_worth: i64,
}

#[derive(SimpleObject)]
pub struct CategoryComparison {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub current: i64,
    pub previous: i64,
    pub average: i64,
}

#[derive(SimpleObject)]
pub struct MonthSummary {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
    pub net: i64,
    pub pending_income: i64,
    pub pending_outcome: i64,
    pub expected_income: i64,
    pub expected_outcome: i64,
    pub expected_net: i64,
    pub previous_income: i64,
    pub previous_outcome: i64,
    pub average_income: i64,
    pub average_outcome: i64,
    pub categories: Vec<CategoryComparison>,
}

#[derive(SimpleObject)]
pub struct BalanceSummary {
    pub total: i64,
    pub available: i64,
    pub projected: i64,
    pub debt: i64,
    pub net_worth: i64,
}

#[derive(SimpleObject)]
pub struct MonthComparison {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub first: i64,
    pub second: i64,
    pub difference: i64,
}

#[derive(SimpleObject)]
pub struct Alert {
    pub key: String,
    pub kind: String,
    pub title: String,
    pub message: String,
    pub date: Option<NaiveDate>,
}
