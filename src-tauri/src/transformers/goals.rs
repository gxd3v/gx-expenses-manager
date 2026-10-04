use chrono::NaiveDate;

use crate::graphql::goals::types;
use crate::models;

impl types::Goal {
    pub fn from_model(
        goal: models::Goal,
        today: NaiveDate,
        projected_date: Option<NaiveDate>,
    ) -> Self {
        Self {
            remaining: goal.remaining(),
            progress: goal.progress(),
            monthly_needed: goal.monthly_needed(today),
            projected_date,
            id: goal.id,
            name: goal.name,
            account_id: goal.account_id,
            account_name: goal.account_name,
            target_amount: goal.target_amount,
            target_date: goal.target_date,
            archived_at: goal.archived_at,
            current_amount: goal.current_amount,
        }
    }
}

impl From<types::GoalInput> for models::GoalInput {
    fn from(input: types::GoalInput) -> Self {
        Self {
            name: input.name,
            account_id: input.account_id,
            target_amount: input.target_amount,
            target_date: input.target_date,
        }
    }
}
