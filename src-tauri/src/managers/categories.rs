use chrono::Utc;
use uuid::Uuid;

use super::{archived_at, optional, required};
use crate::errors::AppError;
use crate::models::{Category, CategoryInput};
use crate::repositories::categories::CategoriesRepository;

#[derive(Clone)]
pub struct CategoriesManager {
    repository: CategoriesRepository,
}

impl CategoriesManager {
    pub fn new(repository: CategoriesRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self, include_archived: bool) -> Result<Vec<Category>, AppError> {
        self.repository.list(include_archived).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Category, AppError> {
        self.repository.get(id).await
    }

    pub async fn create(&self, input: CategoryInput) -> Result<Category, AppError> {
        let input = self.validate(None, input).await?;
        let id = Uuid::now_v7();
        self.repository.insert(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn update(&self, id: Uuid, input: CategoryInput) -> Result<Category, AppError> {
        let input = self.validate(Some(id), input).await?;
        self.repository.update(id, &input, Utc::now()).await?;
        self.repository.get(id).await
    }

    pub async fn set_archived(&self, id: Uuid, archived: bool) -> Result<Category, AppError> {
        self.repository
            .set_archived(id, archived_at(archived), Utc::now())
            .await?;
        self.repository.get(id).await
    }

    pub async fn delete(&self, id: Uuid, reassign_to: Option<Uuid>) -> Result<(), AppError> {
        if self.repository.has_children(id).await? {
            return Err(AppError::conflict(
                "a categoria tem subcategorias, que têm de ser eliminadas ou movidas primeiro",
            ));
        }
        if reassign_to == Some(id) {
            return Err(AppError::validation(
                "a categoria de destino tem de ser diferente",
            ));
        }
        if reassign_to.is_none() && self.repository.in_use(id).await? {
            return Err(AppError::conflict(
                "a categoria está em uso; é necessária uma categoria de destino para os movimentos",
            ));
        }
        if let Some(target) = reassign_to {
            self.repository.get(target).await?;
        }
        self.repository.delete(id, reassign_to).await
    }

    async fn validate(
        &self,
        id: Option<Uuid>,
        input: CategoryInput,
    ) -> Result<CategoryInput, AppError> {
        let input = CategoryInput {
            name: required(&input.name, "o nome é obrigatório")?,
            icon: optional(input.icon),
            color: optional(input.color),
            ..input
        };

        let Some(parent_id) = input.parent_id else {
            return Ok(input);
        };
        if Some(parent_id) == id {
            return Err(AppError::validation(
                "uma categoria não pode ser subcategoria de si própria",
            ));
        }
        if self.repository.get(parent_id).await?.parent_id.is_some() {
            return Err(AppError::validation(
                "só é suportado um nível de subcategorias",
            ));
        }
        if let Some(id) = id
            && self.repository.has_children(id).await?
        {
            return Err(AppError::validation(
                "uma categoria com subcategorias não pode ser subcategoria",
            ));
        }
        Ok(input)
    }
}
