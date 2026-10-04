use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::CategoryKind")]
pub enum CategoryKind {
    Income,
    Outcome,
    Both,
}

#[derive(SimpleObject)]
pub struct Category {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub kind: CategoryKind,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub transaction_count: i64,
}

#[derive(InputObject)]
pub struct CategoryInput {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub kind: CategoryKind,
    pub icon: Option<String>,
    pub color: Option<String>,
}
