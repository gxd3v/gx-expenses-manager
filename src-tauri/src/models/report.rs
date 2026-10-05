use std::cmp::Reverse;

use chrono::{Datelike, Days, NaiveDate};
use uuid::Uuid;

string_enum!(CategoryGrouping {
    Parent => "parent",
    Leaf => "leaf",
});

#[derive(Debug, Clone)]
pub struct MonthlyTotal {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
}

#[derive(Debug, Clone)]
pub struct CategoryAmount {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub amount: i64,
}

#[derive(Debug, Clone)]
pub struct BalancePoint {
    pub month: NaiveDate,
    pub balance: i64,
    pub debt: i64,
}

#[derive(Debug, Clone)]
pub struct CategoryComparison {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub current: i64,
    pub previous: i64,
    pub average: i64,
}

#[derive(Debug, Clone)]
pub struct MonthSummary {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
    pub pending_income: i64,
    pub pending_outcome: i64,
    pub previous_income: i64,
    pub previous_outcome: i64,
    pub average_income: i64,
    pub average_outcome: i64,
    pub categories: Vec<CategoryComparison>,
}

#[derive(Debug, Clone)]
pub struct BalanceSummary {
    pub total: i64,
    pub available: i64,
    pub projected: i64,
    pub credit_debt: i64,
    pub card_debt: i64,
}

#[derive(Debug, Clone)]
pub struct MonthComparison {
    pub category_id: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub first: i64,
    pub second: i64,
}

string_enum!(RecordPeriod {
    AllTime => "all_time",
    Year => "year",
    Month => "month",
    Week => "week",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BalanceMark {
    pub date: NaiveDate,
    pub balance: i64,
}

#[derive(Debug, Clone)]
pub struct BalanceRecord {
    pub period: RecordPeriod,
    pub high: BalanceMark,
    pub low: BalanceMark,
}

pub fn week_start(date: NaiveDate, first_day_of_week: u8) -> NaiveDate {
    let offset = (date.weekday().num_days_from_sunday() + 7 - u32::from(first_day_of_week)) % 7;
    date - Days::new(u64::from(offset))
}

pub fn balance_record(
    period: RecordPeriod,
    opening: i64,
    changes: &[(NaiveDate, i64)],
    from: NaiveDate,
) -> BalanceRecord {
    let (before, during): (Vec<(NaiveDate, i64)>, Vec<_>) =
        changes.iter().partition(|(date, _)| *date < from);
    let mut balance = opening + before.iter().map(|(_, amount)| amount).sum::<i64>();
    let mut marks = Vec::with_capacity(during.len() + 1);
    if !before.is_empty() || during.is_empty() {
        marks.push(BalanceMark {
            date: from,
            balance,
        });
    }
    for (date, amount) in during {
        balance += amount;
        marks.push(BalanceMark { date, balance });
    }

    let high = marks
        .iter()
        .max_by_key(|m| (m.balance, Reverse(m.date)))
        .copied()
        .expect("at least one mark");
    let low = marks
        .iter()
        .min_by_key(|m| (m.balance, m.date))
        .copied()
        .expect("at least one mark");
    BalanceRecord { period, high, low }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn records_track_running_balance() {
        let changes = [
            (date(2025, 1, 10), 500),
            (date(2025, 6, 1), -900),
            (date(2026, 2, 3), 2_000),
            (date(2026, 2, 20), -300),
        ];

        let all = balance_record(RecordPeriod::AllTime, 1_000, &changes, date(2025, 1, 10));
        assert_eq!(
            all.high,
            BalanceMark {
                date: date(2026, 2, 3),
                balance: 2_600
            }
        );
        assert_eq!(
            all.low,
            BalanceMark {
                date: date(2025, 6, 1),
                balance: 600
            }
        );

        let year = balance_record(RecordPeriod::Year, 1_000, &changes, date(2026, 1, 1));
        assert_eq!(
            year.low,
            BalanceMark {
                date: date(2026, 1, 1),
                balance: 600
            }
        );

        let week = balance_record(RecordPeriod::Week, 1_000, &changes, date(2026, 3, 2));
        assert_eq!(week.high, week.low);
        assert_eq!(week.high.balance, 2_300);
    }

    #[test]
    fn week_start_honours_first_day() {
        assert_eq!(week_start(date(2026, 10, 4), 1), date(2026, 9, 28));
        assert_eq!(week_start(date(2026, 10, 4), 0), date(2026, 10, 4));
        assert_eq!(week_start(date(2026, 10, 1), 6), date(2026, 9, 26));
    }
}
