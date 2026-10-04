use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

string_enum!(TransactionKind {
    Income => "income",
    Outcome => "outcome",
    Transfer => "transfer",
});

string_enum!(EntryKind {
    Income => "income",
    Outcome => "outcome",
});

impl EntryKind {
    pub fn signed(self, amount: i64) -> i64 {
        match self {
            Self::Income => amount,
            Self::Outcome => -amount,
        }
    }

    pub fn transaction_kind(self) -> TransactionKind {
        match self {
            Self::Income => TransactionKind::Income,
            Self::Outcome => TransactionKind::Outcome,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub account_name: String,
    pub currency: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub kind: TransactionKind,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
    pub notes: Option<String>,
    pub confirmed: bool,
    pub transfer_id: Option<Uuid>,
    pub recurrence_id: Option<Uuid>,
    pub one_off: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub counterpart_account_id: Option<Uuid>,
    pub counterpart_account_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub kind: TransactionKind,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
    pub notes: Option<String>,
    pub confirmed: bool,
    pub transfer_id: Option<Uuid>,
    pub recurrence_id: Option<Uuid>,
    pub one_off: bool,
}

#[derive(Debug, Clone)]
pub struct TransactionInput {
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub kind: EntryKind,
    pub amount: i64,
    pub date: NaiveDate,
    pub description: String,
    pub notes: Option<String>,
    pub confirmed: bool,
    pub one_off: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    pub account_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub kind: Option<TransactionKind>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub search: Option<String>,
    pub min_amount: Option<i64>,
    pub max_amount: Option<i64>,
    pub confirmed: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TransactionPage {
    pub items: Vec<Transaction>,
    pub total_count: i64,
    pub income: i64,
    pub outcome: i64,
}

impl TransactionRecord {
    pub fn from_input(id: Uuid, input: &TransactionInput) -> Self {
        Self {
            id,
            account_id: input.account_id,
            category_id: input.category_id,
            kind: input.kind.transaction_kind(),
            amount: input.kind.signed(input.amount),
            date: input.date,
            description: input.description.clone(),
            notes: input.notes.clone(),
            confirmed: input.confirmed,
            transfer_id: None,
            recurrence_id: None,
            one_off: input.one_off,
        }
    }
}
