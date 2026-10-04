use chrono::{Days, NaiveDate};
use uuid::Uuid;

use super::recurrences::RecurrencesManager;
use crate::errors::AppError;
use crate::models::dates::{add_months, month_end, month_start};
use crate::models::{
    AccountBalance, Adjustment, ForecastEvent, ForecastInput, ForecastMethod, ForecastMonth,
    Occurrence, TransactionKind, VariableAverage, project,
};
use crate::repositories::accounts::AccountsRepository;
use crate::repositories::credits::CreditsRepository;
use crate::repositories::goals::GoalsRepository;
use crate::repositories::reports::ReportsRepository;
use crate::repositories::transactions::TransactionsRepository;

pub const MAX_MONTHS: u32 = 120;

#[derive(Debug, Clone)]
pub struct RecurrenceChange {
    pub recurrence_id: Uuid,
    pub amount: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ForecastRequest {
    pub months: u32,
    pub method: ForecastMethod,
    pub history_months: u32,
    pub adjustments: Vec<Adjustment>,
    pub recurrence_changes: Vec<RecurrenceChange>,
}

#[derive(Debug, Clone)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct Forecast {
    pub months: Vec<ForecastMonth>,
    pub goals_reached: Vec<Milestone>,
    pub credits_paid: Vec<Milestone>,
}

#[derive(Clone)]
pub struct ForecastsManager {
    accounts: AccountsRepository,
    transactions: TransactionsRepository,
    recurrences: RecurrencesManager,
    reports: ReportsRepository,
    goals: GoalsRepository,
    credits: CreditsRepository,
}

impl ForecastsManager {
    pub fn new(
        accounts: AccountsRepository,
        transactions: TransactionsRepository,
        recurrences: RecurrencesManager,
        reports: ReportsRepository,
        goals: GoalsRepository,
        credits: CreditsRepository,
    ) -> Self {
        Self {
            accounts,
            transactions,
            recurrences,
            reports,
            goals,
            credits,
        }
    }

    pub async fn forecast(
        &self,
        today: NaiveDate,
        request: &ForecastRequest,
    ) -> Result<Forecast, AppError> {
        if !(1..=MAX_MONTHS).contains(&request.months) {
            return Err(AppError::validation(
                "o horizonte tem de estar entre 1 e 120 meses",
            ));
        }

        let end = month_end(add_months(month_start(today), request.months as i32 - 1));
        let balances = self
            .accounts
            .list(today, false)
            .await?
            .into_iter()
            .map(|a| AccountBalance {
                account_id: a.id,
                balance: a.balance,
            })
            .collect();

        let mut events = self.transaction_events(today, end).await?;
        events.extend(
            self.recurrence_events(today, end, &request.recurrence_changes)
                .await?,
        );
        events.extend(request.adjustments.iter().flat_map(Adjustment::events));

        let averages = match request.method {
            ForecastMethod::History => self.averages(today, request.history_months).await?,
            ForecastMethod::Recurring => Vec::new(),
        };

        let months = project(&ForecastInput {
            today,
            months: request.months,
            balances,
            events,
            averages,
        });
        Ok(Forecast {
            goals_reached: self.goals_reached(today, &months).await?,
            credits_paid: self.credits_paid(today, end).await?,
            months,
        })
    }

    async fn transaction_events(
        &self,
        today: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ForecastEvent>, AppError> {
        Ok(self
            .transactions
            .future(today)
            .await?
            .into_iter()
            .filter(|t| t.date <= end)
            .map(|t| ForecastEvent {
                account_id: t.account_id,
                date: t.date,
                amount: t.amount,
                transfer: t.kind == TransactionKind::Transfer,
            })
            .collect())
    }

    async fn recurrence_events(
        &self,
        today: NaiveDate,
        end: NaiveDate,
        changes: &[RecurrenceChange],
    ) -> Result<Vec<ForecastEvent>, AppError> {
        let from = today.checked_add_days(Days::new(1)).unwrap_or(today);
        let occurrences = self.recurrences.occurrences(from, end, None).await?;

        Ok(occurrences
            .into_iter()
            .filter_map(|occurrence| {
                let change = changes
                    .iter()
                    .find(|c| c.recurrence_id == occurrence.recurrence_id);
                let amount = match change {
                    Some(change) => change.amount?,
                    None => occurrence.amount,
                };
                Some(recurrence_events(&occurrence, amount))
            })
            .flatten()
            .collect())
    }

    async fn averages(
        &self,
        today: NaiveDate,
        history_months: u32,
    ) -> Result<Vec<VariableAverage>, AppError> {
        let history = history_months.clamp(1, 24) as i64;
        let current = month_start(today);
        let from = add_months(current, -(history as i32));
        let rows = self.reports.variable_totals(from, current, today).await?;

        Ok(rows
            .into_iter()
            .map(|row| VariableAverage {
                account_id: Uuid::from(row.account_id),
                income: row.income / history,
                outcome: row.outcome / history,
                current_income: row.current_income,
                current_outcome: row.current_outcome,
            })
            .collect())
    }

    async fn goals_reached(
        &self,
        today: NaiveDate,
        months: &[ForecastMonth],
    ) -> Result<Vec<Milestone>, AppError> {
        let goals = self.goals.list(today, false).await?;
        Ok(goals
            .into_iter()
            .filter_map(|goal| {
                let month = months.iter().find(|m| {
                    m.balances
                        .iter()
                        .any(|b| b.account_id == goal.account_id && b.balance >= goal.target_amount)
                })?;
                Some(Milestone {
                    id: goal.id,
                    name: goal.name,
                    date: month.month,
                })
            })
            .collect())
    }

    async fn credits_paid(
        &self,
        today: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Milestone>, AppError> {
        let credits = self.credits.list(false).await?;
        Ok(credits
            .into_iter()
            .filter_map(|credit| {
                let date = credit
                    .summary(today)
                    .projected_end_date
                    .filter(|d| *d <= end)?;
                Some(Milestone {
                    id: credit.id,
                    name: credit.name,
                    date,
                })
            })
            .collect())
    }
}

fn recurrence_events(occurrence: &Occurrence, amount: i64) -> Vec<ForecastEvent> {
    let event = |account_id, amount, transfer| ForecastEvent {
        account_id,
        date: occurrence.date,
        amount,
        transfer,
    };
    match occurrence.to_account_id {
        Some(to_account_id) => vec![
            event(occurrence.account_id, -amount, true),
            event(to_account_id, amount, true),
        ],
        None => vec![event(
            occurrence.account_id,
            occurrence.kind.signed(amount),
            false,
        )],
    }
}
