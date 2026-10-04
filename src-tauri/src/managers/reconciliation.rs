use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::dates::{add_months, month_end, month_start};
use crate::models::{ForgottenCandidate, Reconciliation, ReconciliationStatus};
use crate::repositories::reconciliations::ReconciliationsRepository;

const HISTORY_MONTHS: i32 = 4;

#[derive(Clone)]
pub struct ReconciliationManager {
    repository: ReconciliationsRepository,
}

impl ReconciliationManager {
    pub fn new(repository: ReconciliationsRepository) -> Self {
        Self { repository }
    }

    pub async fn status(
        &self,
        account_id: Uuid,
        date: NaiveDate,
    ) -> Result<ReconciliationStatus, AppError> {
        self.repository.status(account_id, date).await
    }

    pub async fn reconcile(
        &self,
        account_id: Uuid,
        date: NaiveDate,
        statement_balance: i64,
        confirm_from: Option<NaiveDate>,
    ) -> Result<Reconciliation, AppError> {
        if let Some(from) = confirm_from {
            self.repository
                .confirm_period(account_id, from, date)
                .await?;
        }

        let status = self.repository.status(account_id, date).await?;
        let reconciliation = Reconciliation {
            id: Uuid::now_v7(),
            account_id,
            date,
            statement_balance,
            calculated_balance: status.calculated_balance,
            created_at: Utc::now(),
        };
        self.repository.insert(&reconciliation).await?;
        Ok(reconciliation)
    }

    pub async fn history(&self, account_id: Uuid) -> Result<Vec<Reconciliation>, AppError> {
        self.repository.list(account_id).await
    }

    pub async fn forgotten(
        &self,
        account_id: Uuid,
        month: NaiveDate,
    ) -> Result<Vec<ForgottenCandidate>, AppError> {
        let start = month_start(month);
        let history_from = add_months(start, -HISTORY_MONTHS);
        self.repository
            .forgotten(account_id, history_from, start, month_end(start))
            .await
    }
}
