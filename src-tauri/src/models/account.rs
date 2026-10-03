use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    Bank,
    Savings,
    Card,
    Cash,
    Other,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    pub initial_balance: i64,
    pub color: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AccountInput {
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    pub initial_balance: i64,
    pub color: Option<String>,
}

impl AccountKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bank => "bank",
            Self::Savings => "savings",
            Self::Card => "card",
            Self::Cash => "cash",
            Self::Other => "other",
        }
    }
}

impl FromStr for AccountKind {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "bank" => Ok(Self::Bank),
            "savings" => Ok(Self::Savings),
            "card" => Ok(Self::Card),
            "cash" => Ok(Self::Cash),
            "other" => Ok(Self::Other),
            _ => Err(AppError::Corrupted(format!("unknown account kind '{value}'"))),
        }
    }
}
