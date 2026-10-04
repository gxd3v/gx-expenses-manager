use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::errors::AppError;
use crate::models::{Credit, CreditPayment, Frequency};
use crate::repositories::to_id;

#[derive(Debug, FromRow)]
pub struct CreditRow {
    pub id: Hyphenated,
    pub name: String,
    pub institution: Option<String>,
    pub principal: i64,
    pub opening_balance: i64,
    pub annual_rate: f64,
    pub installment: i64,
    pub unit: String,
    pub interval: i64,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub installments: Option<i64>,
    pub account_id: Option<Hyphenated>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub principal_paid: i64,
    pub interest_paid: i64,
    pub payments_count: i64,
}

#[derive(Debug, FromRow)]
pub struct PaymentRow {
    pub id: Hyphenated,
    pub credit_id: Hyphenated,
    pub transaction_id: Option<Hyphenated>,
    pub date: NaiveDate,
    pub principal: i64,
    pub interest: i64,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<CreditRow> for Credit {
    type Error = AppError;

    fn try_from(row: CreditRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from(row.id),
            name: row.name,
            institution: row.institution,
            principal: row.principal,
            opening_balance: row.opening_balance,
            annual_rate: row.annual_rate,
            installment: row.installment,
            frequency: Frequency::new(row.unit.parse()?, row.interval)?,
            start_date: row.start_date,
            end_date: row.end_date,
            installments: row.installments,
            account_id: to_id(row.account_id),
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            principal_paid: row.principal_paid,
            interest_paid: row.interest_paid,
            payments_count: row.payments_count,
        })
    }
}

impl From<PaymentRow> for CreditPayment {
    fn from(row: PaymentRow) -> Self {
        Self {
            id: Uuid::from(row.id),
            credit_id: Uuid::from(row.credit_id),
            transaction_id: to_id(row.transaction_id),
            date: row.date,
            principal: row.principal,
            interest: row.interest,
            created_at: row.created_at,
        }
    }
}
