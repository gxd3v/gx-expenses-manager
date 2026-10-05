use chrono::NaiveDate;

use crate::graphql::purchases::types;
use crate::managers::purchases::PurchaseRequest;
use crate::models;

impl From<types::PurchasePlanInput> for PurchaseRequest {
    fn from(input: types::PurchasePlanInput) -> Self {
        Self {
            account_id: input.account_id,
            amount: input.amount,
            margin: input.margin,
            allow_overdraft: input.allow_overdraft,
            months: input.months,
        }
    }
}

impl From<models::PurchasePlan> for types::PurchasePlan {
    fn from(plan: models::PurchasePlan) -> Self {
        Self {
            balance: plan.balance,
            floor: plan.floor,
            lowest_if_today: plan.lowest_if_today,
            earliest_date: plan.earliest,
            months: plan
                .months
                .into_iter()
                .map(|m| types::PlanMonth {
                    month: m.month,
                    lowest: m.lowest,
                    lowest_after: m.lowest_after,
                })
                .collect(),
        }
    }
}

impl types::WishlistItem {
    pub fn from_model(item: models::WishlistItem, planned_date: Option<NaiveDate>) -> Self {
        Self {
            id: item.id,
            name: item.name,
            amount: item.amount,
            account_id: item.account_id,
            account_name: item.account_name,
            priority: item.priority,
            notes: item.notes,
            purchased_at: item.purchased_at,
            planned_date,
        }
    }
}

impl From<types::WishlistInput> for models::WishlistInput {
    fn from(input: types::WishlistInput) -> Self {
        Self {
            name: input.name,
            amount: input.amount,
            account_id: input.account_id,
            priority: input.priority,
            notes: input.notes,
        }
    }
}
