use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::errors::AppError;
use crate::models::Category;
use crate::repositories::to_id;

#[derive(Debug, FromRow)]
pub struct CategoryRow {
    pub id: Hyphenated,
    pub parent_id: Option<Hyphenated>,
    pub name: String,
    pub kind: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub transaction_count: i64,
}

impl TryFrom<CategoryRow> for Category {
    type Error = AppError;

    fn try_from(row: CategoryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from(row.id),
            parent_id: to_id(row.parent_id),
            name: row.name,
            kind: row.kind.parse()?,
            icon: row.icon,
            color: row.color,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            transaction_count: row.transaction_count,
        })
    }
}
