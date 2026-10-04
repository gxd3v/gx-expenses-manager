use std::collections::HashMap;

use chrono::{Days, NaiveDate, Utc};
use uuid::Uuid;

use super::{credits::CreditsManager, positive, required};
use crate::errors::AppError;
use crate::models::MAX_OCCURRENCE_SHIFT_DAYS;
use crate::models::dates::add_months;
use crate::models::{
    EntryKind, Occurrence, OccurrenceOverride, OccurrenceStatus, Recurrence, RecurrenceInput,
    TransactionRecord, TransferInput,
};
use crate::repositories::recurrences::RecurrencesRepository;

type Overrides = HashMap<Uuid, HashMap<NaiveDate, OccurrenceOverride>>;

#[derive(Clone)]
pub struct RecurrencesManager {
    repository: RecurrencesRepository,
    credits: CreditsManager,
}

impl RecurrencesManager {
    pub fn new(repository: RecurrencesRepository, credits: CreditsManager) -> Self {
        Self {
            repository,
            credits,
        }
    }

    pub async fn list(&self) -> Result<Vec<Recurrence>, AppError> {
        self.repository.list().await
    }

    pub async fn get(&self, id: Uuid) -> Result<Recurrence, AppError> {
        self.repository.get(id).await
    }

    pub async fn next_dates(&self, today: NaiveDate) -> Result<HashMap<Uuid, NaiveDate>, AppError> {
        let occurrences = self.occurrences(today, add_months(today, 24), None).await?;
        let mut next = HashMap::new();
        for occurrence in occurrences {
            next.entry(occurrence.recurrence_id)
                .and_modify(|date: &mut NaiveDate| *date = (*date).min(occurrence.date))
                .or_insert(occurrence.date);
        }
        Ok(next)
    }

    pub async fn occurrences(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        recurrence_id: Option<Uuid>,
    ) -> Result<Vec<Occurrence>, AppError> {
        let overrides = self.grouped_overrides().await?;
        let empty = HashMap::new();
        let mut occurrences: Vec<Occurrence> = self
            .repository
            .list()
            .await?
            .iter()
            .filter(|r| r.is_active() && recurrence_id.is_none_or(|id| id == r.id))
            .flat_map(|r| r.occurrences(from, to, overrides.get(&r.id).unwrap_or(&empty)))
            .collect();

        occurrences.sort_by_key(|o| o.date);
        Ok(occurrences)
    }

    pub async fn create(&self, input: RecurrenceInput) -> Result<Recurrence, AppError> {
        let input = normalize(input)?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, None, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn update(&self, id: Uuid, input: RecurrenceInput) -> Result<Recurrence, AppError> {
        let input = normalize(input)?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn pause(&self, id: Uuid) -> Result<Recurrence, AppError> {
        self.repository
            .set_paused(id, Some(Utc::now()), Utc::now())
            .await?;
        self.repository.get(id).await
    }

    pub async fn resume(&self, today: NaiveDate, id: Uuid) -> Result<Recurrence, AppError> {
        let recurrence = self.repository.get(id).await?;
        let Some(paused_at) = recurrence.paused_at else {
            return Ok(recurrence);
        };

        let overrides = self
            .grouped_overrides()
            .await?
            .remove(&id)
            .unwrap_or_default();
        for occurrence in recurrence.occurrences(paused_at.date_naive(), today, &overrides) {
            self.repository
                .upsert_override(&change(&occurrence, OccurrenceStatus::Skipped, None, None))
                .await?;
        }

        self.repository.set_paused(id, None, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn end(&self, id: Uuid, end_date: NaiveDate) -> Result<Recurrence, AppError> {
        let recurrence = self.repository.get(id).await?;
        if end_date < recurrence.start_date {
            return Err(AppError::validation(
                "a data de fim tem de ser depois da data de início",
            ));
        }
        self.repository.set_end(id, end_date, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }

    pub async fn skip(&self, id: Uuid, occurrence_date: NaiveDate) -> Result<(), AppError> {
        let occurrence = self.pending_occurrence(id, occurrence_date).await?;
        self.repository
            .upsert_override(&change(&occurrence, OccurrenceStatus::Skipped, None, None))
            .await
    }

    pub async fn modify(
        &self,
        id: Uuid,
        occurrence_date: NaiveDate,
        amount: Option<i64>,
        date: Option<NaiveDate>,
    ) -> Result<(), AppError> {
        if let Some(amount) = amount {
            positive(amount, "o valor tem de ser maior que zero")?;
        }
        let max_shift = i64::try_from(MAX_OCCURRENCE_SHIFT_DAYS).unwrap_or(i64::MAX);
        if date.is_some_and(|d| (d - occurrence_date).num_days().abs() > max_shift) {
            return Err(AppError::validation(
                "uma ocorrência só pode ser movida até 31 dias",
            ));
        }

        let occurrence = self.pending_occurrence(id, occurrence_date).await?;
        self.repository
            .upsert_override(&change(
                &occurrence,
                OccurrenceStatus::Pending,
                amount,
                date,
            ))
            .await
    }

    pub async fn reset(&self, id: Uuid, occurrence_date: NaiveDate) -> Result<(), AppError> {
        self.repository.delete_override(id, occurrence_date).await
    }

    pub async fn materialize_due(&self, today: NaiveDate) -> Result<u32, AppError> {
        let overrides = self.grouped_overrides().await?;
        let empty = HashMap::new();
        let mut count = 0;

        for recurrence in self
            .repository
            .list()
            .await?
            .iter()
            .filter(|r| r.is_active())
        {
            let from = recurrence
                .start_date
                .max(recurrence.created_at.date_naive());
            let due = recurrence.occurrences(
                from,
                today,
                overrides.get(&recurrence.id).unwrap_or(&empty),
            );
            for occurrence in due {
                self.materialize(&occurrence).await?;
                count += 1;
            }
        }
        Ok(count)
    }

    async fn materialize(&self, occurrence: &Occurrence) -> Result<(), AppError> {
        if let Some(to_account_id) = occurrence.to_account_id {
            let input = TransferInput {
                from_account_id: occurrence.account_id,
                to_account_id,
                amount: occurrence.amount,
                date: occurrence.date,
                description: occurrence.description.clone(),
            };
            let done = change(occurrence, OccurrenceStatus::Done, None, None);
            return self
                .repository
                .materialize_transfer(Uuid::now_v7(), &input, &done, Utc::now())
                .await;
        }

        let record = TransactionRecord {
            id: Uuid::now_v7(),
            account_id: occurrence.account_id,
            category_id: occurrence.category_id,
            kind: occurrence.kind.transaction_kind(),
            amount: occurrence.kind.signed(occurrence.amount),
            date: occurrence.date,
            description: occurrence.description.clone(),
            notes: None,
            confirmed: false,
            transfer_id: None,
            recurrence_id: Some(occurrence.recurrence_id),
        };

        let payment = match occurrence.credit_id {
            Some(credit_id) => {
                self.credits
                    .payment_for(credit_id, record.id, record.date, occurrence.amount)
                    .await
            }
            None => None,
        };

        let done = OccurrenceOverride {
            transaction_id: Some(record.id),
            ..change(occurrence, OccurrenceStatus::Done, None, None)
        };
        self.repository
            .materialize(&record, &done, payment.as_ref(), Utc::now())
            .await
    }

    async fn pending_occurrence(
        &self,
        id: Uuid,
        occurrence_date: NaiveDate,
    ) -> Result<Occurrence, AppError> {
        let recurrence = self.repository.get(id).await?;
        let overrides = self
            .grouped_overrides()
            .await?
            .remove(&id)
            .unwrap_or_default();
        let margin = Days::new(MAX_OCCURRENCE_SHIFT_DAYS);
        let from = occurrence_date
            .checked_sub_days(margin)
            .unwrap_or(occurrence_date);
        let to = occurrence_date
            .checked_add_days(margin)
            .unwrap_or(occurrence_date);

        recurrence
            .occurrences(from, to, &overrides)
            .into_iter()
            .find(|o| o.occurrence_date == occurrence_date)
            .ok_or_else(|| AppError::validation("ocorrência inexistente ou já tratada"))
    }

    async fn grouped_overrides(&self) -> Result<Overrides, AppError> {
        let mut grouped: Overrides = HashMap::new();
        for change in self.repository.overrides().await? {
            grouped
                .entry(change.recurrence_id)
                .or_default()
                .insert(change.occurrence_date, change);
        }
        Ok(grouped)
    }
}

fn change(
    occurrence: &Occurrence,
    status: OccurrenceStatus,
    amount: Option<i64>,
    date: Option<NaiveDate>,
) -> OccurrenceOverride {
    OccurrenceOverride {
        recurrence_id: occurrence.recurrence_id,
        occurrence_date: occurrence.occurrence_date,
        status,
        amount,
        date,
        transaction_id: None,
    }
}

fn normalize(input: RecurrenceInput) -> Result<RecurrenceInput, AppError> {
    positive(input.amount, "o valor tem de ser maior que zero")?;
    if input.end_date.is_some_and(|end| end < input.start_date) {
        return Err(AppError::validation(
            "a data de fim tem de ser depois da data de início",
        ));
    }
    let description = required(&input.description, "a descrição é obrigatória")?;
    match input.to_account_id {
        Some(to) if to == input.account_id => Err(AppError::validation(
            "as contas de origem e destino têm de ser diferentes",
        )),
        Some(_) => Ok(RecurrenceInput {
            description,
            kind: EntryKind::Outcome,
            category_id: None,
            ..input
        }),
        None => Ok(RecurrenceInput {
            description,
            ..input
        }),
    }
}
