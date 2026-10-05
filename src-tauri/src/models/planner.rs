use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanDay {
    pub date: NaiveDate,
    pub balance: i64,
    pub lowest_ahead: i64,
}

pub fn plan_days(
    today: NaiveDate,
    end: NaiveDate,
    balance: i64,
    events: &[(NaiveDate, i64)],
) -> Vec<PlanDay> {
    let mut running = balance;
    let mut days: Vec<PlanDay> = today
        .iter_days()
        .take_while(|date| *date <= end)
        .map(|date| {
            running += events
                .iter()
                .filter(|(day, _)| *day == date)
                .map(|(_, amount)| amount)
                .sum::<i64>();
            PlanDay {
                date,
                balance: running,
                lowest_ahead: running,
            }
        })
        .collect();

    let mut lowest = i64::MAX;
    for day in days.iter_mut().rev() {
        lowest = lowest.min(day.balance);
        day.lowest_ahead = lowest;
    }
    days
}

pub fn earliest_purchase(days: &[PlanDay], amount: i64, floor: i64) -> Option<NaiveDate> {
    days.iter()
        .find(|day| day.lowest_ahead - amount >= floor)
        .map(|day| day.date)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn salary_and_rent() -> Vec<(NaiveDate, i64)> {
        vec![
            (date(2026, 1, 1), -80_000),
            (date(2026, 1, 28), 150_000),
            (date(2026, 2, 1), -80_000),
            (date(2026, 2, 28), 150_000),
            (date(2026, 3, 1), -80_000),
        ]
    }

    #[test]
    fn lowest_ahead_looks_at_every_future_day() {
        let days = plan_days(
            date(2025, 12, 30),
            date(2026, 3, 5),
            100_000,
            &salary_and_rent(),
        );
        assert_eq!(days[0].balance, 100_000);
        assert_eq!(days[0].lowest_ahead, 20_000);
        let after_salary = days.iter().find(|d| d.date == date(2026, 1, 28)).unwrap();
        assert_eq!(after_salary.balance, 170_000);
        assert_eq!(after_salary.lowest_ahead, 90_000);
    }

    #[test]
    fn earliest_purchase_keeps_future_payments_covered() {
        let days = plan_days(
            date(2025, 12, 30),
            date(2026, 3, 5),
            100_000,
            &salary_and_rent(),
        );
        assert_eq!(
            earliest_purchase(&days, 10_000, 0),
            Some(date(2025, 12, 30))
        );
        assert_eq!(earliest_purchase(&days, 50_000, 0), Some(date(2026, 1, 28)));
        assert_eq!(
            earliest_purchase(&days, 50_000, -60_000),
            Some(date(2025, 12, 30))
        );
        assert_eq!(earliest_purchase(&days, 500_000, 0), None);
    }

    #[test]
    fn planned_purchases_reduce_later_balances() {
        let today = date(2025, 12, 30);
        let mut events = salary_and_rent();
        events.push((today, -30_000));
        let days = plan_days(today, date(2026, 3, 5), 100_000, &events);
        assert_eq!(days[0].balance, 70_000);
        assert_eq!(days[0].lowest_ahead, -10_000);
        assert_eq!(earliest_purchase(&days, 50_000, 0), Some(date(2026, 1, 28)));
    }
}
