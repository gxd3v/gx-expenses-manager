use crate::graphql::accounts::types;
use crate::models;

impl From<models::Account> for types::Account {
    fn from(account: models::Account) -> Self {
        Self {
            id: account.id,
            name: account.name,
            kind: account.kind.into(),
            currency: account.currency,
            initial_balance: account.initial_balance,
            color: account.color,
            icon: account.icon,
            archived_at: account.archived_at,
            created_at: account.created_at,
            updated_at: account.updated_at,
            balance: account.balance,
            available_balance: account.available_balance,
            projected_balance: account.projected_balance,
        }
    }
}

impl From<types::AccountInput> for models::AccountInput {
    fn from(input: types::AccountInput) -> Self {
        Self {
            name: input.name,
            kind: input.kind.into(),
            currency: input.currency,
            initial_balance: input.initial_balance,
            color: input.color,
            icon: input.icon,
        }
    }
}
