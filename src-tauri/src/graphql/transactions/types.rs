use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::graphql::accounts::types::Account;
use crate::graphql::categories::types::Category;
use crate::graphql::credits::types::Credit;
use crate::graphql::goals::types::Goal;
use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::TransactionKind")]
pub enum TransactionKind {
    Income,
    Outcome,
    Transfer,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::EntryKind")]
pub enum EntryKind {
    Income,
    Outcome,
}

#[derive(SimpleObject)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub account_name: String,
    pub currency: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub kind: TransactionKind,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
    pub notes: Option<String>,
    pub confirmed: bool,
    pub transfer_id: Option<Uuid>,
    pub recurrence_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub counterpart_account_id: Option<Uuid>,
    pub counterpart_account_name: Option<String>,
}

#[derive(SimpleObject)]
pub struct TransactionPage {
    pub items: Vec<Transaction>,
    pub total_count: i64,
    pub income: i64,
    pub outcome: i64,
    pub net: i64,
}

#[derive(InputObject)]
pub struct TransactionInput {
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub kind: EntryKind,
    pub amount: i64,
    pub date: NaiveDate,
    #[graphql(default)]
    pub description: String,
    pub notes: Option<String>,
    #[graphql(default)]
    pub confirmed: bool,
}

#[derive(InputObject, Default)]
pub struct TransactionFilter {
    pub account_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub kind: Option<TransactionKind>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub search: Option<String>,
    pub min_amount: Option<i64>,
    pub max_amount: Option<i64>,
    pub confirmed: Option<bool>,
}

#[derive(SimpleObject)]
pub struct Transfer {
    pub id: Uuid,
    pub from_account_id: Uuid,
    pub to_account_id: Uuid,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
}

#[derive(InputObject)]
pub struct TransferInput {
    pub from_account_id: Uuid,
    pub to_account_id: Uuid,
    pub amount: i64,
    pub date: NaiveDate,
    #[graphql(default)]
    pub description: String,
}

#[derive(SimpleObject)]
pub struct SavedFilter {
    pub id: Uuid,
    pub name: String,
    pub filter: String,
}

#[derive(SimpleObject)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub transactions: Vec<Transaction>,
    pub credits: Vec<Credit>,
    pub goals: Vec<Goal>,
}
