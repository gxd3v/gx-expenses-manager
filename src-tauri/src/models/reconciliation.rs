use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Reconciliation {
    pub id: Uuid,
    pub account_id: Uuid,
    pub date: NaiveDate,
    pub statement_balance: i64,
    pub calculated_balance: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ReconciliationStatus {
    pub calculated_balance: i64,
    pub confirmed_balance: i64,
    pub unconfirmed_count: i64,
    pub unconfirmed_total: i64,
}

#[derive(Debug, Clone)]
pub struct ForgottenCandidate {
    pub description: String,
    pub category_name: Option<String>,
    pub average_amount: i64,
    pub months_seen: i64,
    pub last_date: NaiveDate,
}
