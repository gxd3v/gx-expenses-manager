use chrono::{Days, Months, NaiveDate};

use crate::errors::AppError;

string_enum!(FrequencyUnit {
    Day => "day",
    Week => "week",
    Month => "month",
    Year => "year",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frequency {
    pub unit: FrequencyUnit,
    pub interval: u32,
}

impl Frequency {
    pub fn new(unit: FrequencyUnit, interval: i64) -> Result<Self, AppError> {
        match u32::try_from(interval) {
            Ok(interval) if (1..=1000).contains(&interval) => Ok(Self { unit, interval }),
            _ => Err(AppError::Validation(
                "o intervalo tem de estar entre 1 e 1000".into(),
            )),
        }
    }

    pub fn nth(self, start: NaiveDate, n: u32) -> Option<NaiveDate> {
        let steps = n.checked_mul(self.interval)?;
        match self.unit {
            FrequencyUnit::Day => start.checked_add_days(Days::new(steps.into())),
            FrequencyUnit::Week => start.checked_add_days(Days::new(u64::from(steps) * 7)),
            FrequencyUnit::Month => start.checked_add_months(Months::new(steps)),
            FrequencyUnit::Year => start.checked_add_months(Months::new(steps.checked_mul(12)?)),
        }
    }

    pub fn dates(
        self,
        start: NaiveDate,
        end: Option<NaiveDate>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Vec<NaiveDate> {
        let last = end.map_or(to, |end| end.min(to));
        (self.first_index(start, from)..)
            .map_while(|n| self.nth(start, n))
            .take_while(|date| *date <= last)
            .filter(|date| *date >= from)
            .collect()
    }

    pub fn periods_per_year(self) -> f64 {
        let per_year = match self.unit {
            FrequencyUnit::Day => 365.0,
            FrequencyUnit::Week => 52.0,
            FrequencyUnit::Month => 12.0,
            FrequencyUnit::Year => 1.0,
        };
        per_year / f64::from(self.interval)
    }

    fn first_index(self, start: NaiveDate, from: NaiveDate) -> u32 {
        let days = (from - start).num_days();
        let longest_period = i64::from(self.interval)
            * match self.unit {
                FrequencyUnit::Day => 1,
                FrequencyUnit::Week => 7,
                FrequencyUnit::Month => 31,
                FrequencyUnit::Year => 366,
            };
        u32::try_from(days.max(0) / longest_period).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn monthly_dates_clamp_to_month_end() {
        let monthly = Frequency::new(FrequencyUnit::Month, 1).unwrap();
        let dates = monthly.dates(date(2026, 1, 31), None, date(2026, 1, 1), date(2026, 4, 30));
        assert_eq!(
            dates,
            vec![
                date(2026, 1, 31),
                date(2026, 2, 28),
                date(2026, 3, 31),
                date(2026, 4, 30)
            ]
        );
    }

    #[test]
    fn dates_respect_window_and_end() {
        let biweekly = Frequency::new(FrequencyUnit::Week, 2).unwrap();
        let dates = biweekly.dates(
            date(2026, 1, 1),
            Some(date(2026, 2, 20)),
            date(2026, 1, 20),
            date(2026, 12, 31),
        );
        assert_eq!(dates, vec![date(2026, 1, 29), date(2026, 2, 12)]);
    }

    #[test]
    fn far_windows_skip_ahead() {
        let daily = Frequency::new(FrequencyUnit::Day, 1).unwrap();
        let dates = daily.dates(date(2000, 1, 1), None, date(2026, 3, 1), date(2026, 3, 3));
        assert_eq!(dates.len(), 3);
    }

    #[test]
    fn rejects_invalid_interval() {
        assert!(Frequency::new(FrequencyUnit::Day, 0).is_err());
    }
}
