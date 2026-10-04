use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dates::{add_months, month_end, month_start};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastMethod {
    Recurring,
    History,
}

#[derive(Debug, Clone)]
pub struct ForecastEvent {
    pub account_id: Uuid,
    pub date: NaiveDate,
    pub amount: i64,
    pub transfer: bool,
}

#[derive(Debug, Clone)]
pub struct VariableAverage {
    pub account_id: Uuid,
    pub income: i64,
    pub outcome: i64,
    pub current_income: i64,
    pub current_outcome: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountBalance {
    pub account_id: Uuid,
    pub balance: i64,
}

#[derive(Debug, Clone)]
pub struct Adjustment {
    pub account_id: Uuid,
    pub to_account_id: Option<Uuid>,
    pub amount: i64,
    pub date: NaiveDate,
    pub repeat_months: u32,
}

#[derive(Debug, Clone)]
pub struct ForecastInput {
    pub today: NaiveDate,
    pub months: u32,
    pub balances: Vec<AccountBalance>,
    pub events: Vec<ForecastEvent>,
    pub averages: Vec<VariableAverage>,
}

#[derive(Debug, Clone)]
pub struct ForecastMonth {
    pub month: NaiveDate,
    pub income: i64,
    pub outcome: i64,
    pub variable_income: i64,
    pub variable_outcome: i64,
    pub total: i64,
    pub balances: Vec<AccountBalance>,
}

impl VariableAverage {
    fn for_month(&self, current: bool) -> (i64, i64) {
        if !current {
            return (self.income, self.outcome);
        }
        (
            (self.income - self.current_income).max(0),
            (self.outcome - self.current_outcome).max(0),
        )
    }
}

impl Adjustment {
    pub fn events(&self) -> Vec<ForecastEvent> {
        (0..self.repeat_months.max(1) as i32)
            .flat_map(|offset| self.events_at(add_months(self.date, offset)))
            .collect()
    }

    fn events_at(&self, date: NaiveDate) -> Vec<ForecastEvent> {
        let Some(to) = self.to_account_id else {
            return vec![ForecastEvent {
                account_id: self.account_id,
                date,
                amount: self.amount,
                transfer: false,
            }];
        };
        let amount = self.amount.abs();
        vec![
            ForecastEvent {
                account_id: self.account_id,
                date,
                amount: -amount,
                transfer: true,
            },
            ForecastEvent {
                account_id: to,
                date,
                amount,
                transfer: true,
            },
        ]
    }
}

pub fn project(input: &ForecastInput) -> Vec<ForecastMonth> {
    let mut balances = input.balances.clone();
    let start = month_start(input.today);

    (0..input.months as i32)
        .map(|offset| {
            let month = add_months(start, offset);
            let end = month_end(month);
            let mut income = 0;
            let mut outcome = 0;

            let events = input
                .events
                .iter()
                .filter(|e| e.date > input.today && e.date >= month && e.date <= end);
            for event in events {
                apply(&mut balances, event.account_id, event.amount);
                if !event.transfer {
                    income += event.amount.max(0);
                    outcome += (-event.amount).max(0);
                }
            }

            let mut variable_income = 0;
            let mut variable_outcome = 0;
            for average in &input.averages {
                let (avg_income, avg_outcome) = average.for_month(offset == 0);
                apply(&mut balances, average.account_id, avg_income - avg_outcome);
                variable_income += avg_income;
                variable_outcome += avg_outcome;
            }

            ForecastMonth {
                month,
                income: income + variable_income,
                outcome: outcome + variable_outcome,
                variable_income,
                variable_outcome,
                total: balances.iter().map(|b| b.balance).sum(),
                balances: balances.clone(),
            }
        })
        .collect()
}

fn apply(balances: &mut [AccountBalance], account_id: Uuid, amount: i64) {
    if let Some(entry) = balances.iter_mut().find(|b| b.account_id == account_id) {
        entry.balance += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn projects_events_averages_and_transfers() {
        let main = Uuid::from_u128(1);
        let savings = Uuid::from_u128(2);
        let saving_plan = Adjustment {
            account_id: main,
            to_account_id: Some(savings),
            amount: 10_000,
            date: date(2026, 2, 5),
            repeat_months: 2,
        };

        let mut events = vec![
            ForecastEvent {
                account_id: main,
                date: date(2026, 1, 10),
                amount: 999,
                transfer: false,
            },
            ForecastEvent {
                account_id: main,
                date: date(2026, 1, 25),
                amount: 200_000,
                transfer: false,
            },
            ForecastEvent {
                account_id: main,
                date: date(2026, 2, 1),
                amount: -80_000,
                transfer: false,
            },
        ];
        events.extend(saving_plan.events());

        let input = ForecastInput {
            today: date(2026, 1, 20),
            months: 3,
            balances: vec![
                AccountBalance {
                    account_id: main,
                    balance: 100_000,
                },
                AccountBalance {
                    account_id: savings,
                    balance: 0,
                },
            ],
            events,
            averages: vec![VariableAverage {
                account_id: main,
                income: 0,
                outcome: 30_000,
                current_income: 0,
                current_outcome: 20_000,
            }],
        };

        let months = project(&input);
        assert_eq!(months[0].total, 100_000 + 200_000 - 10_000);
        assert_eq!(months[0].income, 200_000);
        assert_eq!(months[1].balances[1].balance, 10_000);
        assert_eq!(months[1].outcome, 80_000 + 30_000);
        assert_eq!(months[1].variable_outcome, 30_000);
        assert_eq!(
            months[2].balances[0].balance,
            290_000 - 110_000 - 10_000 - 30_000 - 10_000
        );
    }
}
