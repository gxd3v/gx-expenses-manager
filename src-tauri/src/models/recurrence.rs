use std::collections::HashMap;

use chrono::{DateTime, Days, NaiveDate, Utc};
use uuid::Uuid;

use super::{EntryKind, Frequency};

pub const MAX_OCCURRENCE_SHIFT_DAYS: u64 = 31;

string_enum!(OccurrenceStatus {
    Pending => "pending",
    Skipped => "skipped",
    Done => "done",
});

#[derive(Debug, Clone)]
pub struct Recurrence {
    pub id: Uuid,
    pub account_id: Uuid,
    pub account_name: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub kind: EntryKind,
    pub amount: i64,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub frequency: Frequency,
    pub paused_at: Option<DateTime<Utc>>,
    pub credit_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub to_account_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RecurrenceInput {
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub kind: EntryKind,
    pub amount: i64,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub frequency: Frequency,
    pub to_account_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct OccurrenceOverride {
    pub recurrence_id: Uuid,
    pub occurrence_date: NaiveDate,
    pub status: OccurrenceStatus,
    pub amount: Option<i64>,
    pub date: Option<NaiveDate>,
    pub transaction_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct Occurrence {
    pub recurrence_id: Uuid,
    pub occurrence_date: NaiveDate,
    pub date: NaiveDate,
    pub amount: i64,
    pub kind: EntryKind,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub description: String,
    pub credit_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub modified: bool,
}

impl Recurrence {
    pub fn is_active(&self) -> bool {
        self.paused_at.is_none()
    }

    pub fn occurrences(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        overrides: &HashMap<NaiveDate, OccurrenceOverride>,
    ) -> Vec<Occurrence> {
        let margin = Days::new(MAX_OCCURRENCE_SHIFT_DAYS);
        let window_start = from.checked_sub_days(margin).unwrap_or(from);
        let window_end = to.checked_add_days(margin).unwrap_or(to);

        self.frequency
            .dates(self.start_date, self.end_date, window_start, window_end)
            .into_iter()
            .filter_map(|date| self.occurrence(date, overrides.get(&date)))
            .filter(|occurrence| occurrence.date >= from && occurrence.date <= to)
            .collect()
    }

    fn occurrence(
        &self,
        date: NaiveDate,
        change: Option<&OccurrenceOverride>,
    ) -> Option<Occurrence> {
        if change.is_some_and(|c| c.status != OccurrenceStatus::Pending) {
            return None;
        }

        Some(Occurrence {
            recurrence_id: self.id,
            occurrence_date: date,
            date: change.and_then(|c| c.date).unwrap_or(date),
            amount: change.and_then(|c| c.amount).unwrap_or(self.amount),
            kind: self.kind,
            account_id: self.account_id,
            category_id: self.category_id,
            description: self.description.clone(),
            credit_id: self.credit_id,
            to_account_id: self.to_account_id,
            modified: change.is_some(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FrequencyUnit;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn rent() -> Recurrence {
        Recurrence {
            id: Uuid::nil(),
            account_id: Uuid::nil(),
            account_name: String::new(),
            category_id: None,
            category_name: None,
            kind: EntryKind::Outcome,
            amount: 80_000,
            description: "Renda".into(),
            start_date: date(2026, 1, 1),
            end_date: None,
            frequency: Frequency::new(FrequencyUnit::Month, 1).unwrap(),
            paused_at: None,
            credit_id: None,
            to_account_id: None,
            to_account_name: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn change(
        day: NaiveDate,
        status: OccurrenceStatus,
        amount: Option<i64>,
        moved: Option<NaiveDate>,
    ) -> OccurrenceOverride {
        OccurrenceOverride {
            recurrence_id: Uuid::nil(),
            occurrence_date: day,
            status,
            amount,
            date: moved,
            transaction_id: None,
        }
    }

    #[test]
    fn applies_skips_and_overrides() {
        let overrides = HashMap::from([
            (
                date(2026, 2, 1),
                change(date(2026, 2, 1), OccurrenceStatus::Skipped, None, None),
            ),
            (
                date(2026, 3, 1),
                change(
                    date(2026, 3, 1),
                    OccurrenceStatus::Pending,
                    Some(90_000),
                    None,
                ),
            ),
            (
                date(2026, 5, 1),
                change(
                    date(2026, 5, 1),
                    OccurrenceStatus::Pending,
                    None,
                    Some(date(2026, 4, 28)),
                ),
            ),
        ]);

        let result = rent().occurrences(date(2026, 1, 1), date(2026, 4, 30), &overrides);
        let summary: Vec<_> = result.iter().map(|o| (o.date, o.amount)).collect();

        assert_eq!(
            summary,
            vec![
                (date(2026, 1, 1), 80_000),
                (date(2026, 3, 1), 90_000),
                (date(2026, 4, 1), 80_000),
                (date(2026, 4, 28), 80_000),
            ]
        );
    }
}
