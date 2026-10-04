use chrono::NaiveDate;

use crate::errors::AppError;
use crate::graphql::recurrences::types;
use crate::models;

impl types::Recurrence {
    pub fn from_model(recurrence: models::Recurrence, next_date: Option<NaiveDate>) -> Self {
        Self {
            id: recurrence.id,
            account_id: recurrence.account_id,
            account_name: recurrence.account_name,
            category_id: recurrence.category_id,
            category_name: recurrence.category_name,
            kind: recurrence.kind.into(),
            amount: recurrence.amount,
            description: recurrence.description,
            start_date: recurrence.start_date,
            end_date: recurrence.end_date,
            unit: recurrence.frequency.unit.into(),
            interval: recurrence.frequency.interval,
            paused_at: recurrence.paused_at,
            credit_id: recurrence.credit_id,
            to_account_id: recurrence.to_account_id,
            to_account_name: recurrence.to_account_name,
            next_date,
        }
    }
}

impl TryFrom<types::RecurrenceInput> for models::RecurrenceInput {
    type Error = AppError;

    fn try_from(input: types::RecurrenceInput) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: input.account_id,
            category_id: input.category_id,
            kind: input.kind.into(),
            amount: input.amount,
            description: input.description,
            start_date: input.start_date,
            end_date: input.end_date,
            frequency: models::Frequency::new(input.unit.into(), input.interval)?,
            to_account_id: input.to_account_id,
        })
    }
}

impl From<models::Occurrence> for types::Occurrence {
    fn from(occurrence: models::Occurrence) -> Self {
        Self {
            recurrence_id: occurrence.recurrence_id,
            occurrence_date: occurrence.occurrence_date,
            date: occurrence.date,
            amount: occurrence.amount,
            kind: occurrence.kind.into(),
            account_id: occurrence.account_id,
            category_id: occurrence.category_id,
            description: occurrence.description,
            credit_id: occurrence.credit_id,
            to_account_id: occurrence.to_account_id,
            modified: occurrence.modified,
        }
    }
}
