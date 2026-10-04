use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use super::{archived_at, optional, positive, required};
use crate::errors::AppError;
use crate::models::{
    BalanceAt, Credit, CreditInput, CreditPayment, EntryKind, PaymentInput, RecurrenceInput,
    Simulation, SimulationInput, TransactionKind, TransactionRecord,
};
use crate::repositories::categories::CategoriesRepository;
use crate::repositories::credits::CreditsRepository;
use crate::repositories::recurrences::RecurrencesRepository;
use crate::repositories::transactions::TransactionsRepository;

const CREDITS_CATEGORY: Uuid = Uuid::from_u128(0x00000000_0000_7000_8000_000000000060);

#[derive(Clone)]
pub struct CreditsManager {
    repository: CreditsRepository,
    recurrences: RecurrencesRepository,
    transactions: TransactionsRepository,
    categories: CategoriesRepository,
}

impl CreditsManager {
    pub fn new(
        repository: CreditsRepository,
        recurrences: RecurrencesRepository,
        transactions: TransactionsRepository,
        categories: CategoriesRepository,
    ) -> Self {
        Self {
            repository,
            recurrences,
            transactions,
            categories,
        }
    }

    pub async fn list(&self, include_archived: bool) -> Result<Vec<Credit>, AppError> {
        self.repository.list(include_archived).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Credit, AppError> {
        self.repository.get(id).await
    }

    pub async fn payments(&self, credit_id: Option<Uuid>) -> Result<Vec<CreditPayment>, AppError> {
        self.repository.payments(credit_id).await
    }

    pub async fn create(
        &self,
        today: NaiveDate,
        input: CreditInput,
        with_recurrence: bool,
    ) -> Result<Credit, AppError> {
        let input = normalize(input)?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, Utc::now()).await?;
        let credit = self.repository.get(id).await?;

        if with_recurrence {
            self.create_recurrence(&credit, today).await?;
        }
        Ok(credit)
    }

    pub async fn update(&self, id: Uuid, input: CreditInput) -> Result<Credit, AppError> {
        let input = normalize(input)?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn set_archived(&self, id: Uuid, archived: bool) -> Result<Credit, AppError> {
        self.repository
            .set_archived(id, archived_at(archived), Utc::now())
            .await?;
        self.repository.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }

    pub async fn simulate(
        &self,
        today: NaiveDate,
        id: Uuid,
        input: SimulationInput,
    ) -> Result<Simulation, AppError> {
        if input.extra_payment < 0 || input.installment.is_some_and(|i| i <= 0) {
            return Err(AppError::validation(
                "os valores da simulação têm de ser positivos",
            ));
        }
        self.repository
            .get(id)
            .await?
            .simulate(today, &input)
            .ok_or_else(|| AppError::validation("o crédito não tem prestações futuras"))
    }

    pub async fn register_payment(&self, input: PaymentInput) -> Result<CreditPayment, AppError> {
        let credit = self.repository.get(input.credit_id).await?;
        let (record, date, amount) = match input.transaction_id {
            Some(transaction_id) => self.linked_transaction(&credit, transaction_id).await?,
            None => (
                Some(self.payment_transaction(&credit, &input).await?),
                input.date,
                input.amount,
            ),
        };

        let (principal, interest) = match (input.principal, input.interest) {
            (Some(principal), Some(interest)) => (principal, interest),
            _ => credit.split(amount),
        };
        if principal < 0 || interest < 0 {
            return Err(AppError::validation(
                "capital e juros não podem ser negativos",
            ));
        }

        let payment = CreditPayment {
            id: Uuid::now_v7(),
            credit_id: credit.id,
            transaction_id: input.transaction_id.or(record.as_ref().map(|r| r.id)),
            date,
            principal,
            interest,
            created_at: Utc::now(),
        };
        self.repository
            .insert_payment(&payment, record.as_ref())
            .await?;
        Ok(payment)
    }

    pub async fn balance_history(
        &self,
        id: Uuid,
        today: NaiveDate,
    ) -> Result<Vec<BalanceAt>, AppError> {
        let credit = self.repository.get(id).await?;
        let payments = self.repository.payments(Some(id)).await?;
        Ok(credit.balance_history(&payments, today))
    }

    pub async fn recurrence_links(&self) -> Result<HashMap<Uuid, Uuid>, AppError> {
        let recurrences = self.recurrences.list().await?;
        Ok(recurrences
            .into_iter()
            .filter_map(|r| Some((r.credit_id?, r.id)))
            .collect())
    }

    pub async fn add_recurrence(&self, id: Uuid, today: NaiveDate) -> Result<(), AppError> {
        if self.recurrence_links().await?.contains_key(&id) {
            return Err(AppError::conflict(
                "este crédito já tem uma recorrência associada",
            ));
        }
        let credit = self.repository.get(id).await?;
        self.create_recurrence(&credit, today).await
    }

    pub async fn delete_payment(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete_payment(id).await
    }

    pub async fn payment_for(
        &self,
        credit_id: Uuid,
        transaction_id: Uuid,
        date: NaiveDate,
        amount: i64,
    ) -> Option<CreditPayment> {
        let credit = self.repository.get(credit_id).await.ok()?;
        let (principal, interest) = credit.split(amount);
        Some(CreditPayment {
            id: Uuid::now_v7(),
            credit_id,
            transaction_id: Some(transaction_id),
            date,
            principal,
            interest,
            created_at: Utc::now(),
        })
    }

    async fn linked_transaction(
        &self,
        credit: &Credit,
        transaction_id: Uuid,
    ) -> Result<(Option<TransactionRecord>, NaiveDate, i64), AppError> {
        let transaction = self.transactions.get(transaction_id).await?;
        if transaction.kind != TransactionKind::Outcome {
            return Err(AppError::validation(
                "só despesas podem ser associadas a pagamentos",
            ));
        }

        let payments = self.repository.payments(Some(credit.id)).await?;
        if payments
            .iter()
            .any(|p| p.transaction_id == Some(transaction_id))
        {
            return Err(AppError::conflict(
                "este movimento já está associado a um pagamento",
            ));
        }
        Ok((None, transaction.date, -transaction.amount))
    }

    async fn payment_transaction(
        &self,
        credit: &Credit,
        input: &PaymentInput,
    ) -> Result<TransactionRecord, AppError> {
        positive(
            input.amount,
            "o valor do pagamento tem de ser maior que zero",
        )?;
        let account_id = input
            .account_id
            .or(credit.account_id)
            .ok_or_else(|| AppError::validation("indica a conta de onde sai o pagamento"))?;

        Ok(TransactionRecord {
            id: Uuid::now_v7(),
            account_id,
            category_id: self.credits_category().await,
            kind: TransactionKind::Outcome,
            amount: -input.amount,
            date: input.date,
            description: format!("Prestação {}", credit.name),
            notes: None,
            confirmed: false,
            transfer_id: None,
            recurrence_id: None,
            one_off: false,
        })
    }

    async fn create_recurrence(&self, credit: &Credit, today: NaiveDate) -> Result<(), AppError> {
        let account_id = credit.account_id.ok_or_else(|| {
            AppError::validation("associa uma conta de pagamento para criar a recorrência")
        })?;
        let Some(start_date) = credit.summary(today).next_payment_date else {
            return Ok(());
        };

        let input = RecurrenceInput {
            account_id,
            category_id: self.credits_category().await,
            kind: EntryKind::Outcome,
            amount: credit.installment,
            description: format!("Prestação {}", credit.name),
            start_date,
            end_date: credit.end_date,
            frequency: credit.frequency,
            to_account_id: None,
            variable_amount: false,
        };
        self.recurrences
            .insert(Uuid::now_v7(), &input, Some(credit.id), Utc::now())
            .await
    }

    async fn credits_category(&self) -> Option<Uuid> {
        self.categories
            .get(CREDITS_CATEGORY)
            .await
            .ok()
            .map(|c| c.id)
    }
}

fn normalize(input: CreditInput) -> Result<CreditInput, AppError> {
    positive(
        input.principal,
        "o capital inicial tem de ser maior que zero",
    )?;
    positive(input.installment, "a prestação tem de ser maior que zero")?;
    if input.opening_balance < 0 || input.opening_balance > input.principal {
        return Err(AppError::validation(
            "o capital em dívida tem de estar entre 0 e o capital inicial",
        ));
    }
    if !(0.0..100.0).contains(&input.annual_rate) {
        return Err(AppError::validation(
            "a taxa de juro tem de estar entre 0% e 100%",
        ));
    }
    if input.installments.is_some_and(|n| n <= 0) {
        return Err(AppError::validation(
            "o número de prestações tem de ser positivo",
        ));
    }
    if input.end_date.is_some_and(|end| end < input.start_date) {
        return Err(AppError::validation(
            "a data de fim tem de ser depois da data de início",
        ));
    }

    Ok(CreditInput {
        name: required(&input.name, "o nome é obrigatório")?,
        institution: optional(input.institution),
        ..input
    })
}
