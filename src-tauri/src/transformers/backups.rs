use crate::graphql::backups::types;
use crate::managers::backups::{BackupFile, ImportPreview};
use crate::repositories::backups::LoadResult;

impl From<BackupFile> for types::BackupFile {
    fn from(file: BackupFile) -> Self {
        Self {
            path: file.path.to_string_lossy().into_owned(),
            name: file.name,
            created_at: file.created_at,
            size: file.size,
        }
    }
}

impl From<ImportPreview> for types::ImportPreview {
    fn from(preview: ImportPreview) -> Self {
        Self {
            format_version: preview.format_version,
            schema_version: preview.schema_version,
            exported_at: preview.exported_at,
            encrypted: preview.encrypted,
            counts: preview
                .counts
                .into_iter()
                .map(|(table, rows)| types::TableCount { table, rows })
                .collect(),
            conflicts: preview.conflicts,
        }
    }
}

impl From<LoadResult> for types::ImportResult {
    fn from(result: LoadResult) -> Self {
        Self {
            inserted: result.inserted,
            skipped: result.skipped,
        }
    }
}
