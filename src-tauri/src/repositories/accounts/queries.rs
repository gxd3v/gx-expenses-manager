pub const LIST: &str = "SELECT id, name, kind, currency, initial_balance, color, archived_at, created_at, updated_at \
     FROM accounts \
     WHERE ?1 OR archived_at IS NULL \
     ORDER BY name COLLATE NOCASE";

pub const GET: &str = "SELECT id, name, kind, currency, initial_balance, color, archived_at, created_at, updated_at \
     FROM accounts \
     WHERE id = ?1";

pub const INSERT: &str = "INSERT INTO accounts (id, name, kind, currency, initial_balance, color, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)";

pub const UPDATE: &str = "UPDATE accounts \
     SET name = ?2, kind = ?3, currency = ?4, initial_balance = ?5, color = ?6, updated_at = ?7 \
     WHERE id = ?1";

pub const ARCHIVE: &str = "UPDATE accounts \
     SET archived_at = ?2, updated_at = ?2 \
     WHERE id = ?1 AND archived_at IS NULL";
