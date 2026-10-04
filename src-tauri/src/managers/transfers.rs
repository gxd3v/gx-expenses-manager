use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use super::positive;
use crate::errors::AppError;
use crate::models::{Transfer, TransferInput};
use crate::repositories::accounts::AccountsRepository;
use crate::repositories::transfers::TransfersRepository;

#[derive(Clone)]
pub struct TransfersManager {
    repository: TransfersRepository,
    accounts: AccountsRepository,
}

impl TransfersManager {
    pub fn new(repository: TransfersRepository, accounts: AccountsRepository) -> Self {
        Self {
            repository,
            accounts,
        }
    }

    pub async fn get(&self, id: Uuid) -> Result<Transfer, AppError> {
        self.repository.get(id).await
    }

    pub async fn create(
        &self,
        today: NaiveDate,
        input: TransferInput,
    ) -> Result<Transfer, AppError> {
        let input = self.validate(today, input).await?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn update(
        &self,
        today: NaiveDate,
        id: Uuid,
        input: TransferInput,
    ) -> Result<Transfer, AppError> {
        let input = self.validate(today, input).await?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }

    async fn validate(
        &self,
        today: NaiveDate,
        input: TransferInput,
    ) -> Result<TransferInput, AppError> {
        positive(input.amount, "o valor tem de ser maior que zero")?;
        if input.from_account_id == input.to_account_id {
            return Err(AppError::validation(
                "as contas de origem e destino têm de ser diferentes",
            ));
        }

        let from = self.accounts.get(today, input.from_account_id).await?;
        let to = self.accounts.get(today, input.to_account_id).await?;
        if from.currency != to.currency {
            return Err(AppError::validation(
                "transferências entre moedas diferentes ainda não são suportadas",
            ));
        }

        Ok(TransferInput {
            description: input.description.trim().to_string(),
            ..input
        })
    }
}
