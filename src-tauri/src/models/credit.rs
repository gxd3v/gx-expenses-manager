use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use super::Frequency;

const MAX_PERIODS: u32 = 1200;

string_enum!(AmortizationMode {
    ReduceTerm => "reduce_term",
    ReduceInstallment => "reduce_installment",
});

#[derive(Debug, Clone)]
pub struct Credit {
    pub id: Uuid,
    pub name: String,
    pub institution: Option<String>,
    pub principal: i64,
    pub opening_balance: i64,
    pub annual_rate: f64,
    pub installment: i64,
    pub frequency: Frequency,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub installments: Option<i64>,
    pub account_id: Option<Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub principal_paid: i64,
    pub interest_paid: i64,
    pub payments_count: i64,
}

#[derive(Debug, Clone)]
pub struct CreditInput {
    pub name: String,
    pub institution: Option<String>,
    pub principal: i64,
    pub opening_balance: i64,
    pub annual_rate: f64,
    pub installment: i64,
    pub frequency: Frequency,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub installments: Option<i64>,
    pub account_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CreditPayment {
    pub id: Uuid,
    pub credit_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub date: NaiveDate,
    pub principal: i64,
    pub interest: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PaymentInput {
    pub credit_id: Uuid,
    pub date: NaiveDate,
    pub amount: i64,
    pub account_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub principal: Option<i64>,
    pub interest: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub number: u32,
    pub date: NaiveDate,
    pub installment: i64,
    pub principal: i64,
    pub interest: i64,
    pub balance: i64,
}

#[derive(Debug, Clone)]
pub struct ScheduleSummary {
    pub installment: i64,
    pub periods: u32,
    pub total_interest: i64,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct CreditSummary {
    pub remaining: i64,
    pub principal_paid: i64,
    pub interest_paid: i64,
    pub next_payment_date: Option<NaiveDate>,
    pub remaining_installments: u32,
    pub projected_end_date: Option<NaiveDate>,
    pub schedule: Vec<ScheduleEntry>,
}

#[derive(Debug, Clone)]
pub struct SimulationInput {
    pub extra_payment: i64,
    pub installment: Option<i64>,
    pub mode: AmortizationMode,
}

#[derive(Debug, Clone)]
pub struct Simulation {
    pub baseline: ScheduleSummary,
    pub scenario: ScheduleSummary,
    pub interest_saved: i64,
    pub periods_saved: i64,
}

impl Credit {
    pub fn period_rate(&self) -> f64 {
        self.annual_rate / 100.0 / self.frequency.periods_per_year()
    }

    pub fn remaining(&self) -> i64 {
        (self.opening_balance - self.principal_paid).max(0)
    }

    pub fn summary(&self, today: NaiveDate) -> CreditSummary {
        let next_payment_date = self.next_payment_date(today);
        let schedule = next_payment_date
            .map(|date| {
                schedule(
                    self.remaining(),
                    self.period_rate(),
                    self.installment,
                    self.frequency,
                    date,
                )
            })
            .unwrap_or_default();

        CreditSummary {
            remaining: self.remaining(),
            principal_paid: self.principal - self.remaining(),
            interest_paid: self.interest_paid,
            next_payment_date,
            remaining_installments: schedule.len() as u32,
            projected_end_date: schedule.last().map(|entry| entry.date),
            schedule,
        }
    }

    pub fn split(&self, amount: i64) -> (i64, i64) {
        let interest = interest_for(self.remaining(), self.period_rate()).min(amount);
        let principal = (amount - interest).min(self.remaining());
        (principal, interest)
    }

    pub fn simulate(&self, today: NaiveDate, input: &SimulationInput) -> Option<Simulation> {
        let first = self.next_payment_date(today)?;
        let rate = self.period_rate();
        let base = schedule(
            self.remaining(),
            rate,
            self.installment,
            self.frequency,
            first,
        );

        let balance = (self.remaining() - input.extra_payment).max(0);
        let installment = input.installment.unwrap_or(match input.mode {
            AmortizationMode::ReduceTerm => self.installment,
            AmortizationMode::ReduceInstallment => annuity(balance, rate, base.len() as u32),
        });
        let scenario = schedule(balance, rate, installment, self.frequency, first);

        let baseline = summarize(&base, self.installment);
        let scenario = summarize(&scenario, installment);
        Some(Simulation {
            interest_saved: baseline.total_interest - scenario.total_interest,
            periods_saved: i64::from(baseline.periods) - i64::from(scenario.periods),
            baseline,
            scenario,
        })
    }

    fn next_payment_date(&self, today: NaiveDate) -> Option<NaiveDate> {
        let horizon = super::dates::add_months(today, 24);
        self.frequency
            .dates(self.start_date, self.end_date, today, horizon)
            .first()
            .copied()
    }
}

pub fn interest_for(balance: i64, rate: f64) -> i64 {
    (balance as f64 * rate).round() as i64
}

pub fn schedule(
    balance: i64,
    rate: f64,
    installment: i64,
    frequency: Frequency,
    first: NaiveDate,
) -> Vec<ScheduleEntry> {
    let mut entries = Vec::new();
    let mut balance = balance;

    for n in 0..MAX_PERIODS {
        let interest = interest_for(balance, rate);
        let Some(date) = frequency.nth(first, n) else {
            break;
        };
        if balance <= 0 || installment <= interest {
            break;
        }

        let principal = (installment - interest).min(balance);
        balance -= principal;
        entries.push(ScheduleEntry {
            number: n + 1,
            date,
            installment: principal + interest,
            principal,
            interest,
            balance,
        });
    }
    entries
}

pub fn annuity(balance: i64, rate: f64, periods: u32) -> i64 {
    if periods == 0 {
        return balance;
    }
    if rate == 0.0 {
        return (balance as f64 / f64::from(periods)).ceil() as i64;
    }
    let factor = 1.0 - (1.0 + rate).powi(-(periods as i32));
    (balance as f64 * rate / factor).ceil() as i64
}

fn summarize(entries: &[ScheduleEntry], installment: i64) -> ScheduleSummary {
    ScheduleSummary {
        installment,
        periods: entries.len() as u32,
        total_interest: entries.iter().map(|entry| entry.interest).sum(),
        end_date: entries.last().map(|entry| entry.date),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FrequencyUnit;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn monthly() -> Frequency {
        Frequency::new(FrequencyUnit::Month, 1).unwrap()
    }

    fn mortgage() -> Credit {
        Credit {
            id: Uuid::nil(),
            name: "Casa".into(),
            institution: None,
            principal: 10_000_000,
            opening_balance: 10_000_000,
            annual_rate: 3.0,
            installment: annuity(10_000_000, 0.0025, 360),
            frequency: monthly(),
            start_date: date(2026, 1, 15),
            end_date: None,
            installments: Some(360),
            account_id: None,
            archived_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            principal_paid: 0,
            interest_paid: 0,
            payments_count: 0,
        }
    }

    #[test]
    fn annuity_matches_french_system() {
        assert_eq!(annuity(10_000_000, 0.0025, 360), 42_161);
        assert_eq!(annuity(12_000, 0.0, 12), 1_000);
    }

    #[test]
    fn schedule_pays_off_balance() {
        let entries = schedule(10_000_000, 0.0025, 42_161, monthly(), date(2026, 1, 15));
        assert_eq!(entries.len(), 360);
        assert_eq!(entries.last().unwrap().balance, 0);
        assert_eq!(entries[0].interest, 25_000);
        assert_eq!(entries[0].principal, 17_161);
    }

    #[test]
    fn schedule_stops_when_installment_does_not_cover_interest() {
        assert!(schedule(10_000_000, 0.0025, 20_000, monthly(), date(2026, 1, 15)).is_empty());
    }

    #[test]
    fn early_repayment_saves_interest() {
        let input = SimulationInput {
            extra_payment: 2_000_000,
            installment: None,
            mode: AmortizationMode::ReduceTerm,
        };
        let simulation = mortgage().simulate(date(2026, 1, 1), &input).unwrap();
        assert!(simulation.interest_saved > 0);
        assert!(simulation.periods_saved > 0);

        let input = SimulationInput {
            mode: AmortizationMode::ReduceInstallment,
            ..input
        };
        let simulation = mortgage().simulate(date(2026, 1, 1), &input).unwrap();
        assert_eq!(simulation.periods_saved, 0);
        assert!(simulation.scenario.installment < simulation.baseline.installment);
    }

    #[test]
    fn split_separates_interest() {
        assert_eq!(mortgage().split(42_161), (17_161, 25_000));
    }
}
