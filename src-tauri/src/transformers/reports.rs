use crate::graphql::reports::types;
use crate::managers::alerts::Alert;
use crate::models;

impl From<models::MonthlyTotal> for types::MonthlyTotal {
    fn from(total: models::MonthlyTotal) -> Self {
        Self {
            month: total.month,
            income: total.income,
            outcome: total.outcome,
            net: total.income - total.outcome,
        }
    }
}

impl From<models::CategoryAmount> for types::CategoryAmount {
    fn from(amount: models::CategoryAmount) -> Self {
        Self {
            category_id: amount.category_id,
            name: amount.name,
            color: amount.color,
            amount: amount.amount,
        }
    }
}

impl From<models::BalancePoint> for types::BalancePoint {
    fn from(point: models::BalancePoint) -> Self {
        Self {
            month: point.month,
            balance: point.balance,
            debt: point.debt,
            net_worth: point.balance - point.debt,
        }
    }
}

impl From<models::CategoryComparison> for types::CategoryComparison {
    fn from(category: models::CategoryComparison) -> Self {
        Self {
            category_id: category.category_id,
            name: category.name,
            color: category.color,
            current: category.current,
            previous: category.previous,
            average: category.average,
        }
    }
}

impl From<models::MonthSummary> for types::MonthSummary {
    fn from(summary: models::MonthSummary) -> Self {
        Self {
            month: summary.month,
            income: summary.income,
            outcome: summary.outcome,
            net: summary.income - summary.outcome,
            pending_income: summary.pending_income,
            pending_outcome: summary.pending_outcome,
            previous_income: summary.previous_income,
            previous_outcome: summary.previous_outcome,
            average_income: summary.average_income,
            average_outcome: summary.average_outcome,
            categories: summary.categories.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Alert> for types::Alert {
    fn from(alert: Alert) -> Self {
        Self {
            key: alert.key,
            kind: alert.kind.as_str().to_string(),
            title: alert.title,
            message: alert.message,
            date: alert.date,
        }
    }
}
