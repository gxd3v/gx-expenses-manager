use chrono::Utc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Account, AccountInput};
use crate::repositories::accounts::AccountsRepository;

pub struct AccountsManager {
    repository: AccountsRepository,
}

impl AccountsManager {
    pub fn new(repository: AccountsRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self, include_archived: bool) -> Result<Vec<Account>, AppError> {
        self.repository.list(include_archived).await
    }

    pub async fn create(&self, input: AccountInput) -> Result<Account, AppError> {
        let input = normalize(input)?;
        let now = Utc::now();
        let account = Account {
            id: Uuid::now_v7(),
            name: input.name,
            kind: input.kind,
            currency: input.currency,
            initial_balance: input.initial_balance,
            color: input.color,
            archived_at: None,
            created_at: now,
            updated_at: now,
        };

        self.repository.insert(&account).await?;
        Ok(account)
    }

    pub async fn update(&self, id: Uuid, input: AccountInput) -> Result<Account, AppError> {
        let input = normalize(input)?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn archive(&self, id: Uuid) -> Result<Account, AppError> {
        self.repository.archive(id, Utc::now()).await?;
        self.repository.get(id).await
    }
}

fn normalize(input: AccountInput) -> Result<AccountInput, AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }

    let currency = input.currency.trim().to_uppercase();
    if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::Validation("currency must be a 3-letter ISO code".into()));
    }

    let color = input.color.map(|c| c.trim().to_string()).filter(|c| !c.is_empty());

    Ok(AccountInput { name, currency, color, ..input })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AccountKind;

    fn input(name: &str, currency: &str) -> AccountInput {
        AccountInput {
            name: name.into(),
            kind: AccountKind::Bank,
            currency: currency.into(),
            initial_balance: 0,
            color: Some("  ".into()),
        }
    }

    #[test]
    fn normalize_trims_and_uppercases() {
        let result = normalize(input("  Main  ", "eur")).unwrap();
        assert_eq!(result.name, "Main");
        assert_eq!(result.currency, "EUR");
        assert_eq!(result.color, None);
    }

    #[test]
    fn normalize_rejects_invalid_input() {
        assert!(normalize(input("   ", "EUR")).is_err());
        assert!(normalize(input("Main", "EU")).is_err());
        assert!(normalize(input("Main", "E1R")).is_err());
    }
}
