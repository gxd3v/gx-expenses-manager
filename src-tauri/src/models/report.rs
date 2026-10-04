use chrono::NaiveDate;
use uuid::Uuid;

string_enum!(CategoryGrouping {
    Parent => "parent",
    Leaf => "leaf",
});

#[derive(Debug, Clone)]
pub struct MonthlyTotal {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
}

#[derive(Debug, Clone)]
pub struct CategoryAmount {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub amount: i64,
}

#[derive(Debug, Clone)]
pub struct BalancePoint {
    pub month: NaiveDate,
    pub balance: i64,
    pub debt: i64,
}

#[derive(Debug, Clone)]
pub struct CategoryComparison {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub current: i64,
    pub previous: i64,
    pub average: i64,
}

#[derive(Debug, Clone)]
pub struct MonthSummary {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
    pub pending_income: i64,
    pub pending_outcome: i64,
    pub previous_income: i64,
    pub previous_outcome: i64,
    pub average_income: i64,
    pub average_outcome: i64,
    pub categories: Vec<CategoryComparison>,
}

#[derive(Debug, Clone)]
pub struct BalanceSummary {
    pub total: i64,
    pub available: i64,
    pub projected: i64,
    pub debt: i64,
}

#[derive(Debug, Clone)]
pub struct MonthComparison {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub first: i64,
    pub second: i64,
}
