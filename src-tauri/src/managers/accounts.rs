use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use super::{archived_at, currency, optional, required};
use crate::errors::AppError;
use crate::models::{Account, AccountInput};
use crate::repositories::accounts::AccountsRepository;

#[derive(Clone)]
pub struct AccountsManager {
    repository: AccountsRepository,
}

impl AccountsManager {
    pub fn new(repository: AccountsRepository) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        today: NaiveDate,
        include_archived: bool,
    ) -> Result<Vec<Account>, AppError> {
        self.repository.list(today, include_archived).await
    }

    pub async fn get(&self, today: NaiveDate, id: Uuid) -> Result<Account, AppError> {
        self.repository.get(today, id).await
    }

    pub async fn create(&self, today: NaiveDate, input: AccountInput) -> Result<Account, AppError> {
        let input = normalize(input)?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, Utc::now()).await?;
        self.repository.get(today, id).await
    }

    pub async fn update(
        &self,
        today: NaiveDate,
        id: Uuid,
        input: AccountInput,
    ) -> Result<Account, AppError> {
        let input = normalize(input)?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(today, id).await
    }

    pub async fn set_archived(
        &self,
        today: NaiveDate,
        id: Uuid,
        archived: bool,
    ) -> Result<Account, AppError> {
        self.repository
            .set_archived(id, archived_at(archived), Utc::now())
            .await?;
        self.repository.get(today, id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if self.repository.in_use(id).await? {
            return Err(AppError::conflict(
                "a conta tem movimentos associados; arquiva-a em vez de a eliminar",
            ));
        }
        self.repository.delete(id).await
    }
}

fn normalize(input: AccountInput) -> Result<AccountInput, AppError> {
    Ok(AccountInput {
        name: required(&input.name, "o nome é obrigatório")?,
        currency: currency(&input.currency)?,
        color: optional(input.color),
        icon: optional(input.icon),
        ..input
    })
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
            icon: None,
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
