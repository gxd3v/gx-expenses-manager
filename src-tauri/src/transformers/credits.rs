use chrono::NaiveDate;

use crate::errors::AppError;
use crate::graphql::credits::types;
use crate::models;

impl types::Credit {
    pub fn from_model(credit: models::Credit, today: NaiveDate) -> Self {
        let summary = credit.summary(today);
        Self {
            id: credit.id,
            name: credit.name,
            institution: credit.institution,
            principal: credit.principal,
            opening_balance: credit.opening_balance,
            annual_rate: credit.annual_rate,
            installment: credit.installment,
            unit: credit.frequency.unit.into(),
            interval: credit.frequency.interval,
            start_date: credit.start_date,
            end_date: credit.end_date,
            installments: credit.installments,
            account_id: credit.account_id,
            archived_at: credit.archived_at,
            payments_count: credit.payments_count,
            remaining: summary.remaining,
            principal_paid: summary.principal_paid,
            interest_paid: summary.interest_paid,
            next_payment_date: summary.next_payment_date,
            remaining_installments: summary.remaining_installments,
            projected_end_date: summary.projected_end_date,
            schedule: summary.schedule.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<models::ScheduleEntry> for types::ScheduleEntry {
    fn from(entry: models::ScheduleEntry) -> Self {
        Self {
            number: entry.number,
            date: entry.date,
            installment: entry.installment,
            principal: entry.principal,
            interest: entry.interest,
            balance: entry.balance,
        }
    }
}

impl TryFrom<types::CreditInput> for models::CreditInput {
    type Error = AppError;

    fn try_from(input: types::CreditInput) -> Result<Self, Self::Error> {
        Ok(Self {
            name: input.name,
            institution: input.institution,
            principal: input.principal,
            opening_balance: input.opening_balance,
            annual_rate: input.annual_rate,
            installment: input.installment,
            frequency: models::Frequency::new(input.unit.into(), input.interval)?,
            start_date: input.start_date,
            end_date: input.end_date,
            installments: input.installments,
            account_id: input.account_id,
        })
    }
}

impl From<models::CreditPayment> for types::CreditPayment {
    fn from(payment: models::CreditPayment) -> Self {
        Self {
            id: payment.id,
            credit_id: payment.credit_id,
            transaction_id: payment.transaction_id,
            date: payment.date,
            principal: payment.principal,
            interest: payment.interest,
            amount: payment.principal + payment.interest,
        }
    }
}

impl From<types::PaymentInput> for models::PaymentInput {
    fn from(input: types::PaymentInput) -> Self {
        Self {
            credit_id: input.credit_id,
            date: input.date,
            amount: input.amount,
            account_id: input.account_id,
            transaction_id: input.transaction_id,
            principal: input.principal,
            interest: input.interest,
        }
    }
}

impl From<types::SimulationInput> for models::SimulationInput {
    fn from(input: types::SimulationInput) -> Self {
        Self {
            extra_payment: input.extra_payment,
            installment: input.installment,
            mode: input.mode.into(),
        }
    }
}

impl From<models::ScheduleSummary> for types::ScheduleSummary {
    fn from(summary: models::ScheduleSummary) -> Self {
        Self {
            installment: summary.installment,
            periods: summary.periods,
            total_interest: summary.total_interest,
            end_date: summary.end_date,
        }
    }
}

impl From<models::Simulation> for types::Simulation {
    fn from(simulation: models::Simulation) -> Self {
        Self {
            baseline: simulation.baseline.into(),
            scenario: simulation.scenario.into(),
            interest_saved: simulation.interest_saved,
            periods_saved: simulation.periods_saved,
        }
    }
}
