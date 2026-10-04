macro_rules! select {
    () => {
        "SELECT c.id, c.name, c.institution, c.principal, c.opening_balance, c.annual_rate, c.installment, c.unit, c.interval, \
         c.start_date, c.end_date, c.installments, c.account_id, c.archived_at, c.created_at, c.updated_at, \
         COALESCE(SUM(p.principal), 0) AS principal_paid, COALESCE(SUM(p.interest), 0) AS interest_paid, \
         COUNT(p.id) AS payments_count \
         FROM credits c LEFT JOIN credit_payments p ON p.credit_id = c.id "
    };
}

pub const LIST: &str = concat!(
    select!(),
    "WHERE ?1 OR c.archived_at IS NULL GROUP BY c.id ORDER BY c.archived_at IS NOT NULL, c.name COLLATE NOCASE"
);

pub const GET: &str = concat!(select!(), "WHERE c.id = ?1 GROUP BY c.id");

pub const INSERT: &str = "INSERT INTO credits \
     (id, name, institution, principal, opening_balance, annual_rate, installment, unit, interval, start_date, end_date, \
     installments, account_id, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)";

pub const UPDATE: &str = "UPDATE credits \
     SET name = ?2, institution = ?3, principal = ?4, opening_balance = ?5, annual_rate = ?6, installment = ?7, unit = ?8, \
     interval = ?9, start_date = ?10, end_date = ?11, installments = ?12, account_id = ?13, updated_at = ?14 \
     WHERE id = ?1";

pub const SET_ARCHIVED: &str = "UPDATE credits SET archived_at = ?2, updated_at = ?3 WHERE id = ?1";

pub const DELETE: &str = "DELETE FROM credits WHERE id = ?1";

pub const PAYMENTS: &str = "SELECT id, credit_id, transaction_id, date, principal, interest, created_at \
     FROM credit_payments WHERE ?1 IS NULL OR credit_id = ?1 ORDER BY date DESC, created_at DESC";

pub const INSERT_PAYMENT: &str = "INSERT INTO credit_payments (id, credit_id, transaction_id, date, principal, interest, created_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

pub const DELETE_PAYMENT: &str = "DELETE FROM credit_payments WHERE id = ?1";
