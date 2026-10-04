use crate::graphql::templates::types;
use crate::models;

impl From<models::Template> for types::Template {
    fn from(template: models::Template) -> Self {
        Self {
            id: template.id,
            name: template.name,
            account_id: template.account_id,
            account_name: template.account_name,
            category_id: template.category_id,
            category_name: template.category_name,
            kind: template.kind.into(),
            amount: template.amount,
            description: template.description,
        }
    }
}

impl From<types::TemplateInput> for models::TemplateInput {
    fn from(input: types::TemplateInput) -> Self {
        Self {
            name: input.name,
            account_id: input.account_id,
            category_id: input.category_id,
            kind: input.kind.into(),
            amount: input.amount,
            description: input.description,
        }
    }
}
