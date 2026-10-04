use chrono::Utc;
use uuid::Uuid;

use super::required;
use crate::errors::AppError;
use crate::models::{Template, TemplateInput};
use crate::repositories::templates::TemplatesRepository;

#[derive(Clone)]
pub struct TemplatesManager {
    repository: TemplatesRepository,
}

impl TemplatesManager {
    pub fn new(repository: TemplatesRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self) -> Result<Vec<Template>, AppError> {
        self.repository.list().await
    }

    pub async fn create(&self, input: TemplateInput) -> Result<Template, AppError> {
        let input = normalize(input)?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn update(&self, id: Uuid, input: TemplateInput) -> Result<Template, AppError> {
        let input = normalize(input)?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

fn normalize(input: TemplateInput) -> Result<TemplateInput, AppError> {
    if input.amount.is_some_and(|amount| amount <= 0) {
        return Err(AppError::validation("o valor tem de ser maior que zero"));
    }
    Ok(TemplateInput {
        name: required(&input.name, "o nome é obrigatório")?,
        description: input.description.trim().to_string(),
        ..input
    })
}
