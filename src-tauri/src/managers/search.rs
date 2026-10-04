use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use super::required;
use crate::errors::AppError;
use crate::models::{Account, Category, Credit, Goal, Transaction, TransactionFilter};
use crate::repositories::accounts::AccountsRepository;
use crate::repositories::categories::CategoriesRepository;
use crate::repositories::credits::CreditsRepository;
use crate::repositories::goals::GoalsRepository;
use crate::repositories::saved_filters::{SavedFilter, SavedFiltersRepository};
use crate::repositories::transactions::TransactionsRepository;

const TRANSACTION_LIMIT: i64 = 20;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub transactions: Vec<Transaction>,
    pub credits: Vec<Credit>,
    pub goals: Vec<Goal>,
}

#[derive(Clone)]
pub struct SearchManager {
    accounts: AccountsRepository,
    categories: CategoriesRepository,
    transactions: TransactionsRepository,
    credits: CreditsRepository,
    goals: GoalsRepository,
    saved_filters: SavedFiltersRepository,
}

impl SearchManager {
    pub fn new(
        accounts: AccountsRepository,
        categories: CategoriesRepository,
        transactions: TransactionsRepository,
        credits: CreditsRepository,
        goals: GoalsRepository,
        saved_filters: SavedFiltersRepository,
    ) -> Self {
        Self {
            accounts,
            categories,
            transactions,
            credits,
            goals,
            saved_filters,
        }
    }

    pub async fn search(&self, today: NaiveDate, text: &str) -> Result<SearchResult, AppError> {
        let needle = text.trim().to_lowercase();
        if needle.is_empty() {
            return Ok(SearchResult {
                accounts: Vec::new(),
                categories: Vec::new(),
                transactions: Vec::new(),
                credits: Vec::new(),
                goals: Vec::new(),
            });
        }

        let matches = |value: &str| value.to_lowercase().contains(&needle);
        let filter = TransactionFilter {
            search: Some(needle.clone()),
            ..Default::default()
        };

        Ok(SearchResult {
            accounts: self
                .accounts
                .list(today, true)
                .await?
                .into_iter()
                .filter(|a| matches(&a.name))
                .collect(),
            categories: self
                .categories
                .list(true)
                .await?
                .into_iter()
                .filter(|c| matches(&c.name))
                .collect(),
            transactions: self
                .transactions
                .page(&filter, TRANSACTION_LIMIT, 0)
                .await?
                .items,
            credits: self
                .credits
                .list(true)
                .await?
                .into_iter()
                .filter(|c| matches(&c.name) || c.institution.as_deref().is_some_and(matches))
                .collect(),
            goals: self
                .goals
                .list(today, true)
                .await?
                .into_iter()
                .filter(|g| matches(&g.name))
                .collect(),
        })
    }

    pub async fn saved_filters(&self) -> Result<Vec<SavedFilter>, AppError> {
        self.saved_filters.list().await
    }

    pub async fn save_filter(&self, name: &str, filter: String) -> Result<SavedFilter, AppError> {
        serde_json::from_str::<serde_json::Value>(&filter)
            .map_err(|_| AppError::validation("filtro inválido"))?;
        let saved = SavedFilter {
            id: Uuid::now_v7(),
            name: required(name, "o nome é obrigatório")?,
            filter,
            created_at: Utc::now(),
        };
        self.saved_filters.insert(&saved).await?;
        Ok(saved)
    }

    pub async fn delete_filter(&self, id: Uuid) -> Result<(), AppError> {
        self.saved_filters.delete(id).await
    }
}
