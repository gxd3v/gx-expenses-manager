use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct BackupFile {
    pub path: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub size: u64,
}

#[derive(SimpleObject)]
pub struct TableCount {
    pub table: String,
    pub rows: usize,
}

#[derive(SimpleObject)]
pub struct ImportPreview {
    pub format_version: u64,
    pub schema_version: i64,
    pub exported_at: Option<String>,
    pub encrypted: bool,
    pub counts: Vec<TableCount>,
    pub conflicts: u64,
}

#[derive(SimpleObject)]
pub struct ImportResult {
    pub inserted: u64,
    pub skipped: u64,
}
