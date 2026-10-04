use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::EntryKind;

string_enum!(CategoryKind {
    Income => "income",
    Outcome => "outcome",
    Both => "both",
});

impl CategoryKind {
    pub fn accepts(self, kind: EntryKind) -> bool {
        matches!(
            (self, kind),
            (Self::Both, _)
                | (Self::Income, EntryKind::Income)
                | (Self::Outcome, EntryKind::Outcome)
        )
    }
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub kind: CategoryKind,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub transaction_count: i64,
}

#[derive(Debug, Clone)]
pub struct CategoryInput {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub kind: CategoryKind,
    pub icon: Option<String>,
    pub color: Option<String>,
}
