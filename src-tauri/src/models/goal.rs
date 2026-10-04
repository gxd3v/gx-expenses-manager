use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use super::dates::months_between;

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: Uuid,
    pub name: String,
    pub account_id: Uuid,
    pub account_name: String,
    pub target_amount: i64,
    pub target_date: Option<NaiveDate>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub current_amount: i64,
}

#[derive(Debug, Clone)]
pub struct GoalInput {
    pub name: String,
    pub account_id: Uuid,
    pub target_amount: i64,
    pub target_date: Option<NaiveDate>,
}

impl Goal {
    pub fn remaining(&self) -> i64 {
        (self.target_amount - self.current_amount).max(0)
    }

    pub fn progress(&self) -> f64 {
        (self.current_amount.max(0) as f64 / self.target_amount as f64).min(1.0)
    }

    pub fn monthly_needed(&self, today: NaiveDate) -> Option<i64> {
        let months = i64::from(months_between(today, self.target_date?).max(1));
        Some((self.remaining() + months - 1) / months)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_progress_and_monthly_need() {
        let goal = Goal {
            id: Uuid::nil(),
            name: "Fundo".into(),
            account_id: Uuid::nil(),
            account_name: String::new(),
            target_amount: 1_000_000,
            target_date: NaiveDate::from_ymd_opt(2027, 12, 1),
            archived_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            current_amount: 250_000,
        };
        let today = NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();

        assert_eq!(goal.remaining(), 750_000);
        assert_eq!(goal.progress(), 0.25);
        assert_eq!(goal.monthly_needed(today), Some(62_500));
    }
}
