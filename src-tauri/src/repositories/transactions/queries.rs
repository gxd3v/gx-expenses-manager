macro_rules! select {
    () => {
        "SELECT t.id, t.account_id, a.name AS account_name, a.currency, t.category_id, \
         CASE WHEN p.name IS NULL THEN c.name ELSE p.name || ' / ' || c.name END AS category_name, \
         t.kind, t.amount, t.date, t.description, t.notes, t.confirmed, t.transfer_id, t.recurrence_id, t.one_off, t.created_at, t.updated_at, \
         ca.id AS counterpart_account_id, ca.name AS counterpart_account_name \
         FROM transactions t \
         JOIN accounts a ON a.id = t.account_id \
         LEFT JOIN transfers tr ON tr.id = t.transfer_id \
         LEFT JOIN accounts ca ON ca.id = CASE WHEN tr.from_account_id = t.account_id THEN tr.to_account_id ELSE tr.from_account_id END \
         LEFT JOIN categories c ON c.id = t.category_id \
         LEFT JOIN categories p ON p.id = c.parent_id "
    };
}

pub const SELECT: &str = select!();

pub const GET: &str = concat!(select!(), "WHERE t.id = ?1");

pub const LIST_FUTURE: &str = concat!(
    select!(),
    "WHERE t.date > ?1 AND a.archived_at IS NULL ORDER BY t.date"
);

pub const PENDING_CONFIRMATIONS: &str = concat!(
    select!(),
    "WHERE t.confirmed = 0 AND t.date <= ?1 \
     AND t.recurrence_id IN (SELECT id FROM recurrences WHERE variable_amount = 1) ORDER BY t.date"
);

pub const TOTALS: &str = "SELECT COUNT(*) AS total_count, \
     COALESCE(SUM(CASE WHEN t.kind = 'income' THEN t.amount END), 0) AS income, \
     COALESCE(SUM(CASE WHEN t.kind = 'outcome' THEN -t.amount END), 0) AS outcome \
     FROM transactions t ";

pub const ORDER: &str = " ORDER BY t.date DESC, t.created_at DESC";

pub const INSERT: &str = "INSERT INTO transactions \
     (id, account_id, category_id, kind, amount, date, description, notes, confirmed, transfer_id, recurrence_id, one_off, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)";

pub const UPDATE: &str = "UPDATE transactions \
     SET account_id = ?2, category_id = ?3, kind = ?4, amount = ?5, date = ?6, description = ?7, notes = ?8, confirmed = ?9, one_off = ?10, updated_at = ?11 \
     WHERE id = ?1 AND transfer_id IS NULL";

pub const DELETE: &str = "DELETE FROM transactions WHERE id = ?1";

pub const SET_CONFIRMED: &str = "UPDATE transactions SET confirmed = ";
