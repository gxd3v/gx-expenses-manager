use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Transfer {
    pub id: Uuid,
    pub from_account_id: Uuid,
    pub to_account_id: Uuid,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TransferInput {
    pub from_account_id: Uuid,
    pub to_account_id: Uuid,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
}
