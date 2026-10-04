use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::EntryKind;

#[derive(Debug, Clone)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub account_id: Uuid,
    pub account_name: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub kind: EntryKind,
    pub amount: Option<i64>,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TemplateInput {
    pub name: String,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub kind: EntryKind,
    pub amount: Option<i64>,
    pub description: String,
}
