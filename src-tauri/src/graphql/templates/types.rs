use async_graphql::{InputObject, SimpleObject};
use uuid::Uuid;

use crate::graphql::transactions::types::EntryKind;

#[derive(SimpleObject)]
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
}

#[derive(InputObject)]
pub struct TemplateInput {
    pub name: String,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub kind: EntryKind,
    pub amount: Option<i64>,
    #[graphql(default)]
    pub description: String,
}
