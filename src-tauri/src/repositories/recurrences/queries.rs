macro_rules! select {
    () => {
        "SELECT r.id, r.account_id, a.name AS account_name, r.category_id, \
         CASE WHEN p.name IS NULL THEN c.name ELSE p.name || ' / ' || c.name END AS category_name, \
         r.kind, r.amount, r.description, r.start_date, r.end_date, r.unit, r.interval, r.paused_at, r.credit_id, \
         r.created_at, r.updated_at \
         FROM recurrences r \
         JOIN accounts a ON a.id = r.account_id \
         LEFT JOIN categories c ON c.id = r.category_id \
         LEFT JOIN categories p ON p.id = c.parent_id "
    };
}

pub const LIST: &str = concat!(select!(), "ORDER BY r.description COLLATE NOCASE");

pub const GET: &str = concat!(select!(), "WHERE r.id = ?1");

pub const INSERT: &str = "INSERT INTO recurrences \
     (id, account_id, category_id, kind, amount, description, start_date, end_date, unit, interval, credit_id, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)";

pub const UPDATE: &str = "UPDATE recurrences \
     SET account_id = ?2, category_id = ?3, kind = ?4, amount = ?5, description = ?6, start_date = ?7, end_date = ?8, \
     unit = ?9, interval = ?10, updated_at = ?11 \
     WHERE id = ?1";

pub const SET_PAUSED: &str = "UPDATE recurrences SET paused_at = ?2, updated_at = ?3 WHERE id = ?1";

pub const SET_END: &str = "UPDATE recurrences SET end_date = ?2, updated_at = ?3 WHERE id = ?1";

pub const DELETE: &str = "DELETE FROM recurrences WHERE id = ?1";

pub const OVERRIDES: &str = "SELECT recurrence_id, occurrence_date, status, amount, date, transaction_id \
     FROM recurrence_occurrences";

pub const UPSERT_OVERRIDE: &str = "INSERT INTO recurrence_occurrences \
     (recurrence_id, occurrence_date, status, amount, date, transaction_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
     ON CONFLICT (recurrence_id, occurrence_date) DO UPDATE SET \
     status = excluded.status, amount = excluded.amount, date = excluded.date, transaction_id = excluded.transaction_id";

pub const DELETE_OVERRIDE: &str = "DELETE FROM recurrence_occurrences \
     WHERE recurrence_id = ?1 AND occurrence_date = ?2 AND status <> 'done'";
