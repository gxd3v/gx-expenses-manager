use crate::graphql::accounts::types;
use crate::models;

impl From<models::Interest> for types::Interest {
    fn from(interest: models::Interest) -> Self {
        Self {
            period_months: interest.period_months,
            tiers: interest
                .tiers
                .into_iter()
                .map(|t| types::InterestTier {
                    min_balance: t.min_balance,
                    rate: t.rate,
                })
                .collect(),
        }
    }
}

impl From<types::Interest> for models::Interest {
    fn from(interest: types::Interest) -> Self {
        Self {
            period_months: interest.period_months,
            tiers: interest
                .tiers
                .into_iter()
                .map(|t| models::InterestTier {
                    min_balance: t.min_balance,
                    rate: t.rate,
                })
                .collect(),
        }
    }
}

impl From<models::Account> for types::Account {
    fn from(account: models::Account) -> Self {
        Self {
            interest_rate: account
                .interest
                .as_ref()
                .map(|i| i.rate_for(account.balance)),
            estimated_interest: account
                .interest
                .as_ref()
                .map(|i| i.net_per_period(account.balance)),
            interest: account.interest.map(Into::into),
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
            interest: input.interest.map(Into::into),
        }
    }
}
