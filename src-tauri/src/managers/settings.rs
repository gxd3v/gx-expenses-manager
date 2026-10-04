use super::{currency, optional};
use crate::errors::AppError;
use crate::models::Settings;
use crate::repositories::settings::SettingsRepository;

#[derive(Clone)]
pub struct SettingsManager {
    repository: SettingsRepository,
}

impl SettingsManager {
    pub fn new(repository: SettingsRepository) -> Self {
        Self { repository }
    }

    pub async fn get(&self) -> Result<Settings, AppError> {
        self.repository.get().await
    }

    pub async fn update(&self, settings: Settings) -> Result<Settings, AppError> {
        let settings = validate(settings)?;
        self.repository.save(&settings).await?;
        Ok(settings)
    }
}

fn validate(settings: Settings) -> Result<Settings, AppError> {
    let ranges = [
        (
            settings.first_day_of_week as u32,
            0,
            6,
            "o primeiro dia da semana é inválido",
        ),
        (
            settings.backup_keep,
            1,
            100,
            "o número de backups a manter tem de estar entre 1 e 100",
        ),
        (
            settings.backup_frequency_days,
            0,
            365,
            "a frequência de backups tem de estar entre 0 e 365 dias",
        ),
        (
            settings.forecast_history_months,
            1,
            24,
            "o histórico das previsões tem de estar entre 1 e 24 meses",
        ),
        (
            settings.forecast_horizon_months,
            1,
            120,
            "o horizonte das previsões tem de estar entre 1 e 120 meses",
        ),
        (
            settings.notify_days_ahead,
            1,
            60,
            "a antecedência dos avisos tem de estar entre 1 e 60 dias",
        ),
        (
            settings.lock_timeout_minutes,
            0,
            1440,
            "o tempo de bloqueio tem de estar entre 0 e 1440 minutos",
        ),
    ];
    if let Some((_, _, _, message)) = ranges
        .iter()
        .find(|(value, min, max, _)| value < min || value > max)
    {
        return Err(AppError::validation(message));
    }
    if settings.low_balance_threshold < 0 {
        return Err(AppError::validation(
            "o limite de saldo baixo não pode ser negativo",
        ));
    }

    Ok(Settings {
        currency: currency(&settings.currency)?,
        backup_dir: optional(settings.backup_dir),
        ..settings
    })
}
