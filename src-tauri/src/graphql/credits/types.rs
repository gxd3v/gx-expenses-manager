use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::graphql::recurrences::types::FrequencyUnit;
use crate::models;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "models::AmortizationMode")]
pub enum AmortizationMode {
    ReduceTerm,
    ReduceInstallment,
}

#[derive(SimpleObject)]
pub struct ScheduleEntry {
    pub number: u32,
    pub date: NaiveDate,
    pub installment: i64,
    pub principal: i64,
    pub interest: i64,
    pub balance: i64,
}

#[derive(SimpleObject)]
pub struct Credit {
    pub id: Uuid,
    pub name: String,
    pub institution: Option<String>,
    pub principal: i64,
    pub opening_balance: i64,
    pub annual_rate: f64,
    pub installment: i64,
    pub unit: FrequencyUnit,
    pub interval: u32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub installments: Option<i64>,
    pub account_id: Option<Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub payments_count: i64,
    pub remaining: i64,
    pub principal_paid: i64,
    pub interest_paid: i64,
    pub next_payment_date: Option<NaiveDate>,
    pub remaining_installments: u32,
    pub projected_end_date: Option<NaiveDate>,
    pub recurrence_id: Option<Uuid>,
    pub schedule: Vec<ScheduleEntry>,
}

#[derive(SimpleObject)]
pub struct CreditBalance {
    pub date: NaiveDate,
    pub balance: i64,
    pub projected: bool,
}

#[derive(InputObject)]
pub struct CreditInput {
    pub name: String,
    pub institution: Option<String>,
    pub principal: i64,
    pub opening_balance: i64,
    pub annual_rate: f64,
    pub installment: i64,
    pub unit: FrequencyUnit,
    #[graphql(default = 1)]
    pub interval: i64,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub installments: Option<i64>,
    pub account_id: Option<Uuid>,
}

#[derive(SimpleObject)]
pub struct CreditPayment {
    pub id: Uuid,
    pub credit_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub date: NaiveDate,
    pub principal: i64,
    pub interest: i64,
    pub amount: i64,
}

#[derive(InputObject)]
pub struct PaymentInput {
    pub credit_id: Uuid,
    pub date: NaiveDate,
    #[graphql(default)]
    pub amount: i64,
    pub account_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub principal: Option<i64>,
    pub interest: Option<i64>,
}

#[derive(InputObject)]
pub struct SimulationInput {
    #[graphql(default)]
    pub extra_payment: i64,
    pub installment: Option<i64>,
    pub mode: AmortizationMode,
}

#[derive(SimpleObject)]
pub struct ScheduleSummary {
    pub installment: i64,
    pub periods: u32,
    pub total_interest: i64,
    pub end_date: Option<NaiveDate>,
}

#[derive(SimpleObject)]
pub struct Simulation {
    pub baseline: ScheduleSummary,
    pub scenario: ScheduleSummary,
    pub interest_saved: i64,
    pub periods_saved: i64,
}
