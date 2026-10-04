use chrono::{Datelike, Local, Months, NaiveDate};

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("day 1 always exists")
}

pub fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let delta = Months::new(months.unsigned_abs());
    let result = if months >= 0 {
        date.checked_add_months(delta)
    } else {
        date.checked_sub_months(delta)
    };
    result.expect("date within supported range")
}

pub fn month_end(date: NaiveDate) -> NaiveDate {
    add_months(month_start(date), 1)
        .pred_opt()
        .expect("date within supported range")
}

pub fn months_between(from: NaiveDate, to: NaiveDate) -> i32 {
    (to.year() - from.year()) * 12 + to.month() as i32 - from.month() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn month_helpers() {
        assert_eq!(month_start(date(2026, 2, 17)), date(2026, 2, 1));
        assert_eq!(month_end(date(2028, 2, 3)), date(2028, 2, 29));
        assert_eq!(add_months(date(2026, 1, 31), 1), date(2026, 2, 28));
        assert_eq!(add_months(date(2026, 1, 15), -2), date(2025, 11, 15));
        assert_eq!(months_between(date(2025, 11, 30), date(2026, 2, 1)), 3);
    }
}
