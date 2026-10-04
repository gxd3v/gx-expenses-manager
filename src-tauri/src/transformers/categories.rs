use crate::graphql::categories::types;
use crate::models;

impl From<models::Category> for types::Category {
    fn from(category: models::Category) -> Self {
        Self {
            id: category.id,
            parent_id: category.parent_id,
            name: category.name,
            kind: category.kind.into(),
            icon: category.icon,
            color: category.color,
            archived_at: category.archived_at,
            transaction_count: category.transaction_count,
        }
    }
}

impl From<types::CategoryInput> for models::CategoryInput {
    fn from(input: types::CategoryInput) -> Self {
        Self {
            parent_id: input.parent_id,
            name: input.name,
            kind: input.kind.into(),
            icon: input.icon,
            color: input.color,
        }
    }
}
