use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::errors::AppError;
use crate::models::{Account, Interest};

#[derive(Debug, FromRow)]
pub struct AccountRow {
    pub id: Hyphenated,
    pub name: String,
    pub kind: String,
    pub currency: String,
    pub initial_balance: i64,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub interest_period_months: Option<u32>,
    pub overdraft_limit: i64,
    pub balance: i64,
    pub available_balance: i64,
    pub projected_balance: i64,
}

impl TryFrom<AccountRow> for Account {
    type Error = AppError;

    fn try_from(row: AccountRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from(row.id),
            name: row.name,
            kind: row.kind.parse()?,
            currency: row.currency,
            initial_balance: row.initial_balance,
            color: row.color,
            icon: row.icon,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            balance: row.balance,
            available_balance: row.available_balance,
            projected_balance: row.projected_balance,
            interest: row.interest_period_months.map(|period_months| Interest {
                period_months,
                tiers: Vec::new(),
            }),
            overdraft_limit: row.overdraft_limit,
        })
    }
}

#[derive(Debug, FromRow)]
pub struct TierRow {
    pub account_id: Hyphenated,
    pub min_balance: i64,
    pub rate: f64,
}
