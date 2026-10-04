use crate::graphql::reconciliation::types;
use crate::models;

impl From<models::ReconciliationStatus> for types::ReconciliationStatus {
    fn from(status: models::ReconciliationStatus) -> Self {
        Self {
            calculated_balance: status.calculated_balance,
            confirmed_balance: status.confirmed_balance,
            unconfirmed_count: status.unconfirmed_count,
            unconfirmed_total: status.unconfirmed_total,
        }
    }
}

impl From<models::Reconciliation> for types::Reconciliation {
    fn from(reconciliation: models::Reconciliation) -> Self {
        Self {
            id: reconciliation.id,
            account_id: reconciliation.account_id,
            date: reconciliation.date,
            statement_balance: reconciliation.statement_balance,
            calculated_balance: reconciliation.calculated_balance,
            difference: reconciliation.statement_balance - reconciliation.calculated_balance,
            created_at: reconciliation.created_at,
        }
    }
}

impl From<models::ForgottenCandidate> for types::ForgottenCandidate {
    fn from(candidate: models::ForgottenCandidate) -> Self {
        Self {
            description: candidate.description,
            category_name: candidate.category_name,
            average_amount: candidate.average_amount,
            months_seen: candidate.months_seen,
            last_date: candidate.last_date,
        }
    }
}
