use std::fs;
use std::path::{Path, PathBuf};

use age::secrecy::SecretString;
use chrono::{DateTime, Utc};
use serde_json::{Map, Value, json};

use super::transactions::TransactionsManager;
use crate::database;
use crate::errors::AppError;
use crate::models::TransactionFilter;
use crate::repositories::backups::{BackupsRepository, LoadResult, TABLES, rows};
use crate::repositories::settings::SettingsRepository;

const FORMAT: &str = "gx-expenses-backup";
const FORMAT_VERSION: u64 = 1;
const AGE_HEADER: &[u8] = b"age-encryption.org/";
const FILE_PREFIX: &str = "expenses-manager-";
const LEGACY_FILE_PREFIX: &str = "gx-expenses-";
const AUTO_PREFIX: &str = "auto-";

#[derive(Debug, Clone)]
pub struct BackupFile {
    pub path: PathBuf,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct ImportPreview {
    pub format_version: u64,
    pub schema_version: i64,
    pub exported_at: Option<String>,
    pub encrypted: bool,
    pub counts: Vec<(String, usize)>,
    pub conflicts: u64,
}

struct Document {
    format_version: u64,
    schema_version: i64,
    exported_at: Option<String>,
    encrypted: bool,
    tables: Map<String, Value>,
}

#[derive(Clone)]
pub struct BackupsManager {
    repository: BackupsRepository,
    settings: SettingsRepository,
    transactions: TransactionsManager,
    data_dir: PathBuf,
    password: SecretString,
    simulation: bool,
}

impl BackupsManager {
    pub fn new(
        repository: BackupsRepository,
        settings: SettingsRepository,
        transactions: TransactionsManager,
        data_dir: PathBuf,
        password: SecretString,
        simulation: bool,
    ) -> Self {
        Self {
            repository,
            settings,
            transactions,
            data_dir,
            password,
            simulation,
        }
    }

    fn writable(&self) -> Result<(), AppError> {
        match self.simulation {
            true => Err(AppError::validation("indisponível em modo de simulação")),
            false => Ok(()),
        }
    }

    pub async fn export(&self, path: &Path, password: Option<String>) -> Result<(), AppError> {
        self.writable()?;
        let document = json!({
            "format": FORMAT,
            "version": FORMAT_VERSION,
            "schemaVersion": self.repository.schema_version().await?,
            "exportedAt": Utc::now().to_rfc3339(),
            "tables": self.repository.dump().await?,
        });
        let bytes =
            serde_json::to_vec_pretty(&document).map_err(|e| AppError::Corrupted(e.to_string()))?;
        let bytes = match password.filter(|p| !p.is_empty()) {
            Some(password) => encrypt(&bytes, password)?,
            None => bytes,
        };
        fs::write(path, bytes)?;
        Ok(())
    }

    pub async fn export_csv(
        &self,
        path: &Path,
        filter: &TransactionFilter,
    ) -> Result<usize, AppError> {
        let transactions = self.transactions.all(filter).await?;
        let mut writer = csv::Writer::from_path(path).map_err(csv_error)?;
        writer
            .write_record([
                "data",
                "conta",
                "categoria",
                "descricao",
                "tipo",
                "valor",
                "moeda",
                "confirmado",
                "notas",
            ])
            .map_err(csv_error)?;

        for t in &transactions {
            writer
                .write_record([
                    t.date.to_string(),
                    t.account_name.clone(),
                    t.category_name.clone().unwrap_or_default(),
                    t.description.clone(),
                    t.kind.as_str().to_string(),
                    format!("{:.2}", t.amount as f64 / 100.0),
                    t.currency.clone(),
                    t.confirmed.to_string(),
                    t.notes.clone().unwrap_or_default(),
                ])
                .map_err(csv_error)?;
        }
        writer.flush()?;
        Ok(transactions.len())
    }

    pub async fn inspect(
        &self,
        path: &Path,
        password: Option<String>,
    ) -> Result<ImportPreview, AppError> {
        let document = self.read(path, password).await?;
        let counts = TABLES
            .iter()
            .map(|table| Ok((table.to_string(), rows(&document.tables, table)?.len())))
            .collect::<Result<_, AppError>>()?;

        Ok(ImportPreview {
            format_version: document.format_version,
            schema_version: document.schema_version,
            exported_at: document.exported_at,
            encrypted: document.encrypted,
            counts,
            conflicts: self.repository.conflicts(&document.tables).await?,
        })
    }

    pub async fn import(
        &self,
        path: &Path,
        password: Option<String>,
        replace: bool,
    ) -> Result<LoadResult, AppError> {
        self.writable()?;
        let document = self.read(path, password).await?;
        self.create_backup("pre-import-").await?;
        self.repository.load(&document.tables, replace).await
    }

    pub async fn create_backup(&self, prefix: &str) -> Result<BackupFile, AppError> {
        self.writable()?;
        let dir = self.backup_dir().await?;
        fs::create_dir_all(&dir)?;
        let name = format!(
            "{FILE_PREFIX}{prefix}{}.db",
            Utc::now().format("%Y%m%d-%H%M%S-%3f")
        );
        let path = dir.join(&name);
        self.repository.snapshot(&path).await?;
        backup_file(&path)
    }

    pub async fn list(&self) -> Result<Vec<BackupFile>, AppError> {
        let dir = self.backup_dir().await?;
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut files: Vec<BackupFile> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| is_backup(path))
            .map(|path| backup_file(&path))
            .collect::<Result<_, _>>()?;
        files.sort_by_key(|file| std::cmp::Reverse(file.created_at));
        Ok(files)
    }

    pub async fn auto_backup(&self) -> Result<Option<BackupFile>, AppError> {
        let settings = self.settings.get().await?;
        if self.simulation || settings.backup_frequency_days == 0 {
            return Ok(None);
        }

        let latest = self.list().await?.into_iter().next();
        let due = latest.is_none_or(|b| {
            (Utc::now() - b.created_at).num_days() >= settings.backup_frequency_days.into()
        });
        if !due {
            return Ok(None);
        }

        let backup = self.create_backup(AUTO_PREFIX).await?;
        self.prune(settings.backup_keep as usize).await?;
        Ok(Some(backup))
    }

    pub async fn verify(&self, path: &Path, password: Option<String>) -> Result<(), AppError> {
        let password = password.unwrap_or_else(|| expose(&self.password));
        database::verify(path, &password).await
    }

    async fn prune(&self, keep: usize) -> Result<(), AppError> {
        let automatic = self
            .list()
            .await?
            .into_iter()
            .filter(|b| is_automatic(&b.name));
        for backup in automatic.skip(keep) {
            fs::remove_file(backup.path)?;
        }
        Ok(())
    }

    async fn backup_dir(&self) -> Result<PathBuf, AppError> {
        let settings = self.settings.get().await?;
        Ok(settings
            .backup_dir
            .map_or_else(|| self.data_dir.join("backups"), PathBuf::from))
    }

    async fn read(&self, path: &Path, password: Option<String>) -> Result<Document, AppError> {
        let bytes = fs::read(path)?;
        let encrypted = bytes.starts_with(AGE_HEADER);
        let bytes = match (encrypted, password) {
            (false, _) => bytes,
            (true, Some(password)) => decrypt(&bytes, password)?,
            (true, None) => {
                return Err(AppError::InvalidBackup(
                    "o ficheiro está encriptado; password necessária".into(),
                ));
            }
        };

        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| AppError::InvalidBackup("JSON inválido".into()))?;
        let document = parse(migrate(value)?, encrypted)?;
        if document.schema_version > self.repository.schema_version().await? {
            return Err(AppError::InvalidBackup(
                "criado por uma versão mais recente da aplicação".into(),
            ));
        }
        Ok(document)
    }
}

fn parse(value: Value, encrypted: bool) -> Result<Document, AppError> {
    let invalid = |message: &str| AppError::InvalidBackup(message.into());
    if value.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(invalid("formato desconhecido"));
    }

    let tables = value
        .get("tables")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| invalid("sem tabelas"))?;
    if let Some(unknown) = tables.keys().find(|key| !TABLES.contains(&key.as_str())) {
        return Err(AppError::InvalidBackup(format!(
            "tabela desconhecida '{unknown}'"
        )));
    }

    Ok(Document {
        format_version: value
            .get("version")
            .and_then(Value::as_u64)
            .ok_or_else(|| invalid("sem versão"))?,
        schema_version: value
            .get("schemaVersion")
            .and_then(Value::as_i64)
            .ok_or_else(|| invalid("sem versão do esquema"))?,
        exported_at: value
            .get("exportedAt")
            .and_then(Value::as_str)
            .map(str::to_string),
        encrypted,
        tables,
    })
}

fn migrate(value: Value) -> Result<Value, AppError> {
    match value.get("version").and_then(Value::as_u64) {
        Some(FORMAT_VERSION) => Ok(value),
        Some(version) if version > FORMAT_VERSION => Err(AppError::InvalidBackup(
            "formato de backup mais recente do que esta versão da aplicação".into(),
        )),
        _ => Err(AppError::InvalidBackup(
            "versão de formato não suportada".into(),
        )),
    }
}

fn encrypt(bytes: &[u8], password: String) -> Result<Vec<u8>, AppError> {
    let recipient = age::scrypt::Recipient::new(SecretString::from(password));
    age::encrypt(&recipient, bytes).map_err(|e| AppError::Corrupted(e.to_string()))
}

fn decrypt(bytes: &[u8], password: String) -> Result<Vec<u8>, AppError> {
    let identity = age::scrypt::Identity::new(SecretString::from(password));
    age::decrypt(&identity, bytes)
        .map_err(|_| AppError::InvalidBackup("password errada ou ficheiro danificado".into()))
}

fn expose(secret: &SecretString) -> String {
    use age::secrecy::ExposeSecret;
    secret.expose_secret().to_string()
}

fn is_backup(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    [FILE_PREFIX, LEGACY_FILE_PREFIX]
        .iter()
        .any(|prefix| name.starts_with(prefix))
        && name.ends_with(".db")
}

fn is_automatic(name: &str) -> bool {
    [FILE_PREFIX, LEGACY_FILE_PREFIX]
        .iter()
        .any(|prefix| name.starts_with(&format!("{prefix}{AUTO_PREFIX}")))
}

fn backup_file(path: &Path) -> Result<BackupFile, AppError> {
    let metadata = fs::metadata(path)?;
    Ok(BackupFile {
        path: path.to_path_buf(),
        name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string(),
        created_at: metadata.modified().map(DateTime::<Utc>::from)?,
        size: metadata.len(),
    })
}

fn csv_error(error: csv::Error) -> AppError {
    AppError::Io(std::io::Error::other(error.to_string()))
}
