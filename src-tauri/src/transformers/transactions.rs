use crate::graphql::transactions::types;
use crate::models;
use crate::repositories::saved_filters::SavedFilter;

impl From<models::Transaction> for types::Transaction {
    fn from(transaction: models::Transaction) -> Self {
        Self {
            id: transaction.id,
            account_id: transaction.account_id,
            account_name: transaction.account_name,
            currency: transaction.currency,
            category_id: transaction.category_id,
            category_name: transaction.category_name,
            kind: transaction.kind.into(),
            amount: transaction.amount,
            date: transaction.date,
            description: transaction.description,
            notes: transaction.notes,
            confirmed: transaction.confirmed,
            transfer_id: transaction.transfer_id,
            recurrence_id: transaction.recurrence_id,
            one_off: transaction.one_off,
            created_at: transaction.created_at,
            counterpart_account_id: transaction.counterpart_account_id,
            counterpart_account_name: transaction.counterpart_account_name,
        }
    }
}

impl From<models::TransactionPage> for types::TransactionPage {
    fn from(page: models::TransactionPage) -> Self {
        Self {
            items: page.items.into_iter().map(Into::into).collect(),
            total_count: page.total_count,
            income: page.income,
            outcome: page.outcome,
            net: page.income - page.outcome,
        }
    }
}

impl From<types::TransactionInput> for models::TransactionInput {
    fn from(input: types::TransactionInput) -> Self {
        Self {
            account_id: input.account_id,
            category_id: input.category_id,
            kind: input.kind.into(),
            amount: input.amount,
            date: input.date,
            description: input.description,
            notes: input.notes,
            confirmed: input.confirmed,
            one_off: input.one_off,
        }
    }
}

impl From<types::TransactionFilter> for models::TransactionFilter {
    fn from(filter: types::TransactionFilter) -> Self {
        Self {
            account_id: filter.account_id,
            category_id: filter.category_id,
            kind: filter.kind.map(Into::into),
            date_from: filter.date_from,
            date_to: filter.date_to,
            search: filter.search,
            min_amount: filter.min_amount,
            max_amount: filter.max_amount,
            confirmed: filter.confirmed,
        }
    }
}

impl From<models::Transfer> for types::Transfer {
    fn from(transfer: models::Transfer) -> Self {
        Self {
            id: transfer.id,
            from_account_id: transfer.from_account_id,
            to_account_id: transfer.to_account_id,
            amount: transfer.amount,
            date: transfer.date,
            description: transfer.description,
        }
    }
}

impl From<types::TransferInput> for models::TransferInput {
    fn from(input: types::TransferInput) -> Self {
        Self {
            from_account_id: input.from_account_id,
            to_account_id: input.to_account_id,
            amount: input.amount,
            date: input.date,
            description: input.description,
        }
    }
}

impl From<SavedFilter> for types::SavedFilter {
    fn from(filter: SavedFilter) -> Self {
        Self {
            id: filter.id,
            name: filter.name,
            filter: filter.filter,
        }
    }
}
