use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, Days, NaiveDate};
use uuid::Uuid;

use super::recurrences::RecurrencesManager;
use crate::errors::AppError;
use crate::models::dates::{add_months, month_end, month_start, months_between};
use crate::models::{
    Account, AccountKind, BalancePoint, BalanceRecord, BalanceSummary, CategoryAmount,
    CategoryComparison, CategoryGrouping, Credit, CreditPayment, EntryKind, MonthComparison,
    MonthSummary, MonthlyTotal, RecordPeriod, TransactionKind, balance_record, week_start,
};
use crate::repositories::accounts::AccountsRepository;
use crate::repositories::credits::CreditsRepository;
use crate::repositories::reports::{CategoryMonthRow, ReportsRepository, parse_month};
use crate::repositories::to_id;
use crate::repositories::transactions::TransactionsRepository;

const HISTORY_MONTHS: i32 = 6;

#[derive(Clone)]
pub struct ReportsManager {
    repository: ReportsRepository,
    accounts: AccountsRepository,
    credits: CreditsRepository,
    transactions: TransactionsRepository,
    recurrences: RecurrencesManager,
}

impl ReportsManager {
    pub fn new(
        repository: ReportsRepository,
        accounts: AccountsRepository,
        credits: CreditsRepository,
        transactions: TransactionsRepository,
        recurrences: RecurrencesManager,
    ) -> Self {
        Self {
            repository,
            accounts,
            credits,
            transactions,
            recurrences,
        }
    }

    pub async fn monthly_totals(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        account_id: Option<Uuid>,
    ) -> Result<Vec<MonthlyTotal>, AppError> {
        self.totals(from, to, account_id, false).await
    }

    async fn totals(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        account_id: Option<Uuid>,
        regular_only: bool,
    ) -> Result<Vec<MonthlyTotal>, AppError> {
        let (from, to) = (month_start(from), month_end(to));
        let totals = self
            .repository
            .monthly_totals(from, to, account_id, regular_only)
            .await?;
        let by_month: HashMap<NaiveDate, &MonthlyTotal> =
            totals.iter().map(|t| (t.month, t)).collect();

        Ok((0..=months_between(from, to))
            .map(|offset| add_months(from, offset))
            .map(|month| {
                by_month.get(&month).map_or(
                    MonthlyTotal {
                        month,
                        income: 0,
                        outcome: 0,
                    },
                    |t| (*t).clone(),
                )
            })
            .collect())
    }

    pub async fn balance_summary(&self, today: NaiveDate) -> Result<BalanceSummary, AppError> {
        let (accounts, cards): (Vec<_>, Vec<_>) = self
            .accounts
            .list(today, false)
            .await?
            .into_iter()
            .partition(Account::counts_in_total);
        let credits = self.credits.list(false).await?;
        let card_debt: i64 = cards
            .iter()
            .filter(|c| c.kind == AccountKind::CreditCard)
            .map(|c| (-c.balance).max(0))
            .sum();
        Ok(BalanceSummary {
            total: accounts.iter().map(|a| a.balance).sum(),
            available: accounts.iter().map(|a| a.available_balance).sum(),
            projected: accounts.iter().map(|a| a.projected_balance).sum(),
            credit_debt: credits.iter().map(Credit::remaining).sum(),
            card_debt,
        })
    }

    pub async fn category_averages(
        &self,
        today: NaiveDate,
        kind: EntryKind,
        months: u32,
    ) -> Result<Vec<CategoryAmount>, AppError> {
        let months = months.clamp(1, 24);
        let start = month_start(today);
        let to = start.pred_opt().unwrap_or(start);
        let amounts = self
            .repository
            .by_category(
                kind,
                add_months(start, -(months as i32)),
                to,
                CategoryGrouping::Parent,
                None,
            )
            .await?;
        Ok(amounts
            .into_iter()
            .map(|a| CategoryAmount {
                amount: a.amount / i64::from(months),
                ..a
            })
            .collect())
    }

    pub async fn compare_months(
        &self,
        first: NaiveDate,
        second: NaiveDate,
    ) -> Result<Vec<MonthComparison>, AppError> {
        let spending = |month: NaiveDate| {
            self.repository.by_category(
                EntryKind::Outcome,
                month_start(month),
                month_end(month),
                CategoryGrouping::Parent,
                None,
            )
        };
        let (first, second) = (spending(first).await?, spending(second).await?);

        let mut rows: Vec<MonthComparison> = first
            .into_iter()
            .map(|c| MonthComparison {
                category_id: c.category_id,
                name: c.name,
                color: c.color,
                first: c.amount,
                second: 0,
            })
            .collect();
        for category in second {
            match rows
                .iter_mut()
                .find(|r| r.category_id == category.category_id)
            {
                Some(row) => row.second = category.amount,
                None => rows.push(MonthComparison {
                    category_id: category.category_id,
                    name: category.name,
                    color: category.color,
                    first: 0,
                    second: category.amount,
                }),
            }
        }
        rows.sort_by_key(|r| std::cmp::Reverse(r.first.max(r.second)));
        Ok(rows)
    }

    pub async fn by_category(
        &self,
        kind: EntryKind,
        from: NaiveDate,
        to: NaiveDate,
        grouping: CategoryGrouping,
        account_id: Option<Uuid>,
    ) -> Result<Vec<CategoryAmount>, AppError> {
        self.repository
            .by_category(kind, from, to, grouping, account_id)
            .await
    }

    pub async fn balance_history(
        &self,
        today: NaiveDate,
        months: u32,
        account_id: Option<Uuid>,
    ) -> Result<Vec<BalancePoint>, AppError> {
        let accounts: Vec<_> = self
            .accounts
            .list(today, true)
            .await?
            .into_iter()
            .filter(|a| account_id.map_or(a.counts_in_total(), |id| id == a.id))
            .collect();

        let mut by_month: BTreeMap<NaiveDate, i64> = BTreeMap::new();
        for row in self.repository.account_months(today).await? {
            if accounts.iter().any(|a| a.id == Uuid::from(row.account_id)) {
                *by_month.entry(parse_month(&row.month)?).or_default() += row.amount;
            }
        }

        let (credits, payments) = match account_id {
            Some(_) => (Vec::new(), Vec::new()),
            None => (
                self.credits.list(true).await?,
                self.credits.payments(None).await?,
            ),
        };

        let base: i64 = accounts.iter().map(|a| a.initial_balance).sum();
        let first = add_months(month_start(today), 1 - months.clamp(1, 240) as i32);
        Ok((0..=months_between(first, today))
            .map(|offset| add_months(first, offset))
            .map(|month| BalancePoint {
                month,
                balance: base
                    + by_month
                        .range(..=month)
                        .map(|(_, amount)| amount)
                        .sum::<i64>(),
                debt: debt_at(&credits, &payments, month_end(month).min(today)),
            })
            .collect())
    }

    pub async fn balance_records(
        &self,
        today: NaiveDate,
        first_day_of_week: u8,
        account_id: Option<Uuid>,
    ) -> Result<Vec<BalanceRecord>, AppError> {
        let opening: i64 = self
            .accounts
            .list(today, account_id.is_some())
            .await?
            .iter()
            .filter(|a| account_id.map_or(a.counts_in_total(), |id| id == a.id))
            .map(|a| a.initial_balance)
            .sum();
        let changes = self.repository.daily_changes(today, account_id).await?;

        let first = changes.first().map_or(today, |(date, _)| *date);
        let year = today.with_ordinal(1).unwrap_or(today);
        Ok([
            (RecordPeriod::AllTime, first),
            (RecordPeriod::Year, year),
            (RecordPeriod::Month, month_start(today)),
            (RecordPeriod::Week, week_start(today, first_day_of_week)),
        ]
        .into_iter()
        .map(|(period, from)| balance_record(period, opening, &changes, from))
        .collect())
    }

    pub async fn month_summary(
        &self,
        today: NaiveDate,
        month: NaiveDate,
    ) -> Result<MonthSummary, AppError> {
        let start = month_start(month);
        let end = month_end(month);
        let realized_end = end.min(today);
        let previous = add_months(start, -1);
        let history_start = add_months(start, -HISTORY_MONTHS);

        let totals = self.totals(history_start, end, None, true).await?;
        let realized = self
            .repository
            .monthly_totals(start, realized_end, None, false)
            .await?;
        let realized = realized.first().cloned().unwrap_or(MonthlyTotal {
            month: start,
            income: 0,
            outcome: 0,
        });
        let previous_total = totals.iter().find(|t| t.month == previous);
        let history: Vec<_> = totals.iter().filter(|t| t.month < start).collect();

        let (pending_income, pending_outcome) = self.pending(today, start, end).await?;
        let rows = self
            .repository
            .category_months(history_start, realized_end)
            .await?;

        Ok(MonthSummary {
            month: start,
            income: realized.income,
            outcome: realized.outcome,
            pending_income,
            pending_outcome,
            previous_income: previous_total.map_or(0, |t| t.income),
            previous_outcome: previous_total.map_or(0, |t| t.outcome),
            average_income: average(history.iter().map(|t| t.income)),
            average_outcome: average(history.iter().map(|t| t.outcome)),
            categories: compare_categories(&rows, start)?,
        })
    }

    async fn pending(
        &self,
        today: NaiveDate,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<(i64, i64), AppError> {
        let from = start.max(today.checked_add_days(Days::new(1)).unwrap_or(today));
        if from > end {
            return Ok((0, 0));
        }

        let ignored: Vec<Uuid> = self
            .accounts
            .list(today, true)
            .await?
            .into_iter()
            .filter(|a| !a.counts_in_estimates())
            .map(|a| a.id)
            .collect();
        let future = self.transactions.future(today).await?;
        let transactions = future
            .iter()
            .filter(|t| t.kind != TransactionKind::Transfer && t.date >= from && t.date <= end)
            .filter(|t| !ignored.contains(&t.account_id))
            .map(|t| t.amount);
        let occurrences = self.recurrences.occurrences(from, end, None).await?;
        let amounts: Vec<i64> = transactions
            .chain(
                occurrences
                    .iter()
                    .filter(|o| o.to_account_id.is_none() && !ignored.contains(&o.account_id))
                    .map(|o| o.kind.signed(o.amount)),
            )
            .collect();

        Ok((
            amounts.iter().filter(|a| **a > 0).sum(),
            amounts.iter().filter(|a| **a < 0).map(|a| -a).sum(),
        ))
    }
}

fn debt_at(credits: &[Credit], payments: &[CreditPayment], date: NaiveDate) -> i64 {
    credits
        .iter()
        .filter(|c| c.created_at.date_naive() <= date)
        .map(|credit| {
            let paid: i64 = payments
                .iter()
                .filter(|p| p.credit_id == credit.id && p.date <= date)
                .map(|p| p.principal)
                .sum();
            (credit.opening_balance - paid).max(0)
        })
        .sum()
}

fn average(values: impl Iterator<Item = i64>) -> i64 {
    let values: Vec<i64> = values.collect();
    if values.is_empty() {
        return 0;
    }
    values.iter().sum::<i64>() / values.len() as i64
}

fn compare_categories(
    rows: &[CategoryMonthRow],
    month: NaiveDate,
) -> Result<Vec<CategoryComparison>, AppError> {
    let previous = add_months(month, -1);
    let mut categories: Vec<CategoryComparison> = Vec::new();

    for row in rows {
        let category_id = to_id(row.category_id);
        let row_month = parse_month(&row.month)?;
        let index = match categories.iter().position(|c| c.category_id == category_id) {
            Some(index) => index,
            None => {
                categories.push(CategoryComparison {
                    category_id,
                    name: row.name.clone(),
                    color: row.color.clone(),
                    current: 0,
                    previous: 0,
                    average: 0,
                });
                categories.len() - 1
            }
        };

        let entry = &mut categories[index];
        match row_month {
            m if m == month => entry.current += row.amount,
            m if m == previous => {
                entry.previous += row.regular;
                entry.average += row.regular;
            }
            _ => entry.average += row.regular,
        }
    }

    for category in &mut categories {
        category.average /= i64::from(HISTORY_MONTHS);
    }
    categories.sort_by(|a, b| b.current.cmp(&a.current).then(b.average.cmp(&a.average)));
    Ok(categories)
}
