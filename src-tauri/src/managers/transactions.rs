use chrono::Utc;
use uuid::Uuid;

use super::{optional, positive};
use crate::errors::AppError;
use crate::models::{
    Transaction, TransactionFilter, TransactionInput, TransactionPage, TransactionRecord,
};
use crate::repositories::categories::CategoriesRepository;
use crate::repositories::transactions::TransactionsRepository;
use crate::repositories::transfers::TransfersRepository;

const MAX_PAGE_SIZE: i64 = 500;

#[derive(Clone)]
pub struct TransactionsManager {
    repository: TransactionsRepository,
    categories: CategoriesRepository,
    transfers: TransfersRepository,
}

impl TransactionsManager {
    pub fn new(
        repository: TransactionsRepository,
        categories: CategoriesRepository,
        transfers: TransfersRepository,
    ) -> Self {
        Self {
            repository,
            categories,
            transfers,
        }
    }

    pub async fn page(
        &self,
        filter: &TransactionFilter,
        limit: i64,
        offset: i64,
    ) -> Result<TransactionPage, AppError> {
        self.repository
            .page(filter, limit.clamp(1, MAX_PAGE_SIZE), offset.max(0))
            .await
    }

    pub async fn all(&self, filter: &TransactionFilter) -> Result<Vec<Transaction>, AppError> {
        Ok(self.repository.page(filter, i64::MAX, 0).await?.items)
    }

    pub async fn get(&self, id: Uuid) -> Result<Transaction, AppError> {
        self.repository.get(id).await
    }

    pub async fn create(&self, input: TransactionInput) -> Result<Transaction, AppError> {
        let input = self.validate(input).await?;
        let record = TransactionRecord::from_input(Uuid::now_v7(), &input);
        self.repository.insert(&record, Utc::now()).await?;
        self.repository.get(record.id).await
    }

    pub async fn update(&self, id: Uuid, input: TransactionInput) -> Result<Transaction, AppError> {
        let existing = self.repository.get(id).await?;
        if existing.transfer_id.is_some() {
            return Err(AppError::conflict(
                "este movimento pertence a uma transferência; edita a transferência",
            ));
        }

        let input = self.validate(input).await?;
        let record = TransactionRecord {
            recurrence_id: existing.recurrence_id,
            ..TransactionRecord::from_input(id, &input)
        };
        self.repository.update(&record, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        match self.repository.get(id).await?.transfer_id {
            Some(transfer_id) => self.transfers.delete(transfer_id).await,
            None => self.repository.delete(id).await,
        }
    }

    pub async fn set_confirmed(&self, ids: &[Uuid], confirmed: bool) -> Result<u64, AppError> {
        self.repository.set_confirmed(ids, confirmed).await
    }

    async fn validate(&self, input: TransactionInput) -> Result<TransactionInput, AppError> {
        positive(input.amount, "o valor tem de ser maior que zero")?;

        if let Some(category_id) = input.category_id
            && !self
                .categories
                .get(category_id)
                .await?
                .kind
                .accepts(input.kind)
        {
            return Err(AppError::validation(
                "a categoria não é compatível com o tipo de movimento",
            ));
        }

        Ok(TransactionInput {
            description: input.description.trim().to_string(),
            notes: optional(input.notes),
            ..input
        })
    }
}
