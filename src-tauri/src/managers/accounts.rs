use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use super::{archived_at, currency, optional, required};
use crate::errors::AppError;
use crate::models::{Account, AccountInput, Interest};
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
                "a conta tem movimentos associados; só pode ser arquivada",
            ));
        }
        self.repository.delete(id).await
    }
}

fn normalize(input: AccountInput) -> Result<AccountInput, AppError> {
    if let Some(interest) = &input.interest {
        validate_interest(interest)?;
    }
    Ok(AccountInput {
        name: required(&input.name, "o nome é obrigatório")?,
        currency: currency(&input.currency)?,
        color: optional(input.color),
        icon: optional(input.icon),
        ..input
    })
}

fn validate_interest(interest: &Interest) -> Result<(), AppError> {
    if !(1..=120).contains(&interest.period_months) {
        return Err(AppError::validation(
            "o vencimento dos juros tem de estar entre 1 e 120 meses",
        ));
    }
    if interest.tiers.is_empty() {
        return Err(AppError::validation("é necessária pelo menos uma taxa de juro"));
    }
    if interest
        .tiers
        .iter()
        .any(|t| t.min_balance < 0 || !(0.0..=100.0).contains(&t.rate))
    {
        return Err(AppError::validation(
            "as taxas têm de estar entre 0% e 100% e os saldos mínimos não podem ser negativos",
        ));
    }
    let mut minimums: Vec<i64> = interest.tiers.iter().map(|t| t.min_balance).collect();
    minimums.sort_unstable();
    minimums.dedup();
    if minimums.len() != interest.tiers.len() {
        return Err(AppError::validation(
            "não podem existir dois escalões com o mesmo saldo mínimo",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountKind, InterestTier};

    fn input(name: &str, currency: &str) -> AccountInput {
        AccountInput {
            name: name.into(),
            kind: AccountKind::Bank,
            currency: currency.into(),
            initial_balance: 0,
            color: Some("  ".into()),
            icon: None,
            interest: None,
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

    #[test]
    fn normalize_validates_interest_tiers() {
        let tier = |min_balance, rate| InterestTier { min_balance, rate };
        let with = |period_months, tiers| AccountInput {
            interest: Some(Interest {
                period_months,
                tiers,
            }),
            ..input("Fundo", "EUR")
        };
        assert!(normalize(with(3, vec![tier(0, 1.0), tier(500_000, 1.25)])).is_ok());
        assert!(normalize(with(0, vec![tier(0, 1.0)])).is_err());
        assert!(normalize(with(1, Vec::new())).is_err());
        assert!(normalize(with(1, vec![tier(0, 1.0), tier(0, 2.0)])).is_err());
        assert!(normalize(with(1, vec![tier(0, 101.0)])).is_err());
    }
}
