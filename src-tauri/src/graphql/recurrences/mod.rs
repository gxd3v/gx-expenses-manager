pub mod types;

use async_graphql::{Context, Object, Result};
use chrono::NaiveDate;
use uuid::Uuid;

use super::module;
use crate::models::dates::today;
use types::{Occurrence, Recurrence, RecurrenceInput};

#[derive(Default)]
pub struct RecurrencesQuery;

#[Object]
impl RecurrencesQuery {
    async fn recurrences(&self, ctx: &Context<'_>) -> Result<Vec<Recurrence>> {
        let manager = &module(ctx).recurrences;
        let next = manager.next_dates(today()).await?;
        let recurrences = manager.list().await?;
        Ok(recurrences
            .into_iter()
            .map(|r| {
                let next_date = next.get(&r.id).copied();
                Recurrence::from_model(r, next_date)
            })
            .collect())
    }

    async fn occurrences(
        &self,
        ctx: &Context<'_>,
        from: NaiveDate,
        to: NaiveDate,
        recurrence_id: Option<Uuid>,
    ) -> Result<Vec<Occurrence>> {
        let occurrences = module(ctx)
            .recurrences
            .occurrences(from, to, recurrence_id)
            .await?;
        Ok(occurrences.into_iter().map(Occurrence::from).collect())
    }
}

#[derive(Default)]
pub struct RecurrencesMutation;

#[Object]
impl RecurrencesMutation {
    async fn create_recurrence(
        &self,
        ctx: &Context<'_>,
        input: RecurrenceInput,
    ) -> Result<Recurrence> {
        let manager = &module(ctx).recurrences;
        let recurrence = manager.create(input.try_into()?).await?;
        manager.materialize_due(today()).await?;
        Ok(Recurrence::from_model(recurrence, None))
    }

    async fn update_recurrence(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: RecurrenceInput,
    ) -> Result<Recurrence> {
        let manager = &module(ctx).recurrences;
        let recurrence = manager.update(id, input.try_into()?).await?;
        manager.materialize_due(today()).await?;
        Ok(Recurrence::from_model(recurrence, None))
    }

    async fn pause_recurrence(&self, ctx: &Context<'_>, id: Uuid) -> Result<Recurrence> {
        Ok(Recurrence::from_model(
            module(ctx).recurrences.pause(id).await?,
            None,
        ))
    }

    async fn resume_recurrence(&self, ctx: &Context<'_>, id: Uuid) -> Result<Recurrence> {
        let manager = &module(ctx).recurrences;
        let recurrence = manager.resume(today(), id).await?;
        manager.materialize_due(today()).await?;
        Ok(Recurrence::from_model(recurrence, None))
    }

    async fn end_recurrence(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        end_date: NaiveDate,
    ) -> Result<Recurrence> {
        Ok(Recurrence::from_model(
            module(ctx).recurrences.end(id, end_date).await?,
            None,
        ))
    }

    async fn delete_recurrence(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        module(ctx).recurrences.delete(id).await?;
        Ok(true)
    }

    async fn skip_occurrence(
        &self,
        ctx: &Context<'_>,
        recurrence_id: Uuid,
        occurrence_date: NaiveDate,
    ) -> Result<bool> {
        module(ctx)
            .recurrences
            .skip(recurrence_id, occurrence_date)
            .await?;
        Ok(true)
    }

    async fn modify_occurrence(
        &self,
        ctx: &Context<'_>,
        recurrence_id: Uuid,
        occurrence_date: NaiveDate,
        amount: Option<i64>,
        date: Option<NaiveDate>,
    ) -> Result<bool> {
        module(ctx)
            .recurrences
            .modify(recurrence_id, occurrence_date, amount, date)
            .await?;
        Ok(true)
    }

    async fn reset_occurrence(
        &self,
        ctx: &Context<'_>,
        recurrence_id: Uuid,
        occurrence_date: NaiveDate,
    ) -> Result<bool> {
        module(ctx)
            .recurrences
            .reset(recurrence_id, occurrence_date)
            .await?;
        Ok(true)
    }

    async fn materialize_due_occurrences(&self, ctx: &Context<'_>) -> Result<u32> {
        Ok(module(ctx).recurrences.materialize_due(today()).await?)
    }
}
