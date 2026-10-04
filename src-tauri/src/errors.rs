#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("registo não encontrado")]
    NotFound,
    #[error("password incorreta")]
    WrongPassword,
    #[error("a aplicação está bloqueada")]
    Locked,
    #[error("dados inválidos: {0}")]
    Corrupted(String),
    #[error("backup inválido: {0}")]
    InvalidBackup(String),
    #[error("erro de base de dados: {0}")]
    Database(#[from] sqlx::Error),
    #[error("erro ao migrar a base de dados: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("erro de ficheiro: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Tauri(#[from] tauri::Error),
}

impl AppError {
    pub fn validation(message: &str) -> Self {
        Self::Validation(message.into())
    }

    pub fn conflict(message: &str) -> Self {
        Self::Conflict(message.into())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
