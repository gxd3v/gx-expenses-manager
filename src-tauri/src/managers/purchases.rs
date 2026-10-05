use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use super::forecasts::ForecastsManager;
use super::{archived_at, optional, positive, required};
use crate::errors::AppError;
use crate::models::dates::{add_months, month_end, month_start};
use crate::models::{
    Account, PlanDay, PlanMonth, PurchasePlan, WishlistInput, WishlistItem, earliest_purchase,
    plan_days,
};
use crate::repositories::wishlist::WishlistRepository;

const MAX_MONTHS: u32 = 36;
const WISHLIST_MONTHS: u32 = 24;

#[derive(Debug, Clone)]
pub struct PurchaseRequest {
    pub account_id: Uuid,
    pub amount: i64,
    pub margin: i64,
    pub allow_overdraft: bool,
    pub months: u32,
}

#[derive(Clone)]
pub struct PurchasesManager {
    repository: WishlistRepository,
    forecasts: ForecastsManager,
}

impl PurchasesManager {
    pub fn new(repository: WishlistRepository, forecasts: ForecastsManager) -> Self {
        Self {
            repository,
            forecasts,
        }
    }

    pub async fn plan(
        &self,
        today: NaiveDate,
        request: &PurchaseRequest,
    ) -> Result<PurchasePlan, AppError> {
        positive(request.amount, "o valor tem de ser maior que zero")?;
        if !(1..=MAX_MONTHS).contains(&request.months) {
            return Err(AppError::validation(
                "o horizonte tem de estar entre 1 e 36 meses",
            ));
        }
        if request.margin < 0 {
            return Err(AppError::validation(
                "a margem de segurança não pode ser negativa",
            ));
        }

        let end = horizon_end(today, request.months);
        let (account, events) = self
            .forecasts
            .account_timeline(today, request.account_id, end)
            .await?;
        let days = plan_days(today, end, account.balance, &events);
        let floor = floor(&account, request.margin, request.allow_overdraft);

        Ok(PurchasePlan {
            balance: account.balance,
            floor,
            lowest_if_today: days[0].lowest_ahead - request.amount,
            earliest: earliest_purchase(&days, request.amount, floor),
            months: months(&days, request.amount),
        })
    }

    pub async fn wishlist(
        &self,
        today: NaiveDate,
        margin: i64,
        allow_overdraft: bool,
    ) -> Result<Vec<(WishlistItem, Option<NaiveDate>)>, AppError> {
        let end = horizon_end(today, WISHLIST_MONTHS);
        let mut timelines: HashMap<Uuid, (Account, Vec<(NaiveDate, i64)>)> = HashMap::new();
        let mut planned = Vec::new();

        for item in self.repository.list().await? {
            if item.purchased_at.is_some() {
                planned.push((item, None));
                continue;
            }
            if !timelines.contains_key(&item.account_id) {
                let timeline = self
                    .forecasts
                    .account_timeline(today, item.account_id, end)
                    .await?;
                timelines.insert(item.account_id, timeline);
            }
            let (account, events) = timelines
                .get_mut(&item.account_id)
                .expect("timeline loaded above");
            let days = plan_days(today, end, account.balance, events);
            let date = earliest_purchase(
                &days,
                item.amount,
                floor(account, margin.max(0), allow_overdraft),
            );
            if let Some(date) = date {
                events.push((date, -item.amount));
            }
            planned.push((item, date));
        }
        Ok(planned)
    }

    pub async fn create(&self, input: WishlistInput) -> Result<WishlistItem, AppError> {
        let input = normalize(input)?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn update(&self, id: Uuid, input: WishlistInput) -> Result<WishlistItem, AppError> {
        let input = normalize(input)?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn set_purchased(&self, id: Uuid, purchased: bool) -> Result<WishlistItem, AppError> {
        self.repository
            .set_purchased(id, archived_at(purchased), Utc::now())
            .await?;
        self.repository.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

fn horizon_end(today: NaiveDate, months: u32) -> NaiveDate {
    month_end(add_months(month_start(today), months as i32 - 1))
}

fn floor(account: &Account, margin: i64, allow_overdraft: bool) -> i64 {
    match allow_overdraft {
        true => margin - account.overdraft_limit,
        false => margin,
    }
}

fn months(days: &[PlanDay], amount: i64) -> Vec<PlanMonth> {
    let mut months: Vec<PlanMonth> = Vec::new();
    for day in days {
        let month = month_start(day.date);
        match months.last_mut() {
            Some(last) if last.month == month => last.lowest = last.lowest.min(day.balance),
            _ => months.push(PlanMonth {
                month,
                lowest: day.balance,
                lowest_after: day.lowest_ahead - amount,
            }),
        }
    }
    months
}

fn normalize(input: WishlistInput) -> Result<WishlistInput, AppError> {
    positive(input.amount, "o valor tem de ser maior que zero")?;
    if !(1..=3).contains(&input.priority) {
        return Err(AppError::validation("a prioridade é inválida"));
    }
    Ok(WishlistInput {
        name: required(&input.name, "o nome é obrigatório")?,
        notes: optional(input.notes),
        ..input
    })
}
