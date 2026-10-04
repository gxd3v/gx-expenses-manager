macro_rules! select {
    () => {
        "SELECT a.id, a.name, a.kind, a.currency, a.initial_balance, a.color, a.icon, a.archived_at, a.created_at, a.updated_at, a.interest_period_months, \
         a.initial_balance + COALESCE(SUM(CASE WHEN t.date <= ?1 THEN t.amount END), 0) AS balance, \
         a.initial_balance + COALESCE(SUM(CASE WHEN t.date <= ?1 OR t.amount < 0 THEN t.amount END), 0) AS available_balance, \
         a.initial_balance + COALESCE(SUM(t.amount), 0) AS projected_balance \
         FROM accounts a LEFT JOIN transactions t ON t.account_id = a.id "
    };
}

pub const LIST: &str = concat!(
    select!(),
    "WHERE ?2 OR a.archived_at IS NULL GROUP BY a.id ORDER BY a.archived_at IS NOT NULL, a.name COLLATE NOCASE"
);

pub const GET: &str = concat!(select!(), "WHERE a.id = ?2 GROUP BY a.id");

pub const INSERT: &str = "INSERT INTO accounts (id, name, kind, currency, initial_balance, color, icon, interest_period_months, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)";

pub const UPDATE: &str = "UPDATE accounts \
     SET name = ?2, kind = ?3, currency = ?4, initial_balance = ?5, color = ?6, icon = ?7, interest_period_months = ?8, updated_at = ?9 \
     WHERE id = ?1";

pub const SET_ARCHIVED: &str =
    "UPDATE accounts SET archived_at = ?2, updated_at = ?3 WHERE id = ?1";

pub const IN_USE: &str = "SELECT EXISTS (SELECT 1 FROM transactions WHERE account_id = ?1) \
     OR EXISTS (SELECT 1 FROM recurrences WHERE account_id = ?1) \
     OR EXISTS (SELECT 1 FROM credits WHERE account_id = ?1) \
     OR EXISTS (SELECT 1 FROM goals WHERE account_id = ?1) \
     OR EXISTS (SELECT 1 FROM transfers WHERE from_account_id = ?1 OR to_account_id = ?1)";

pub const DELETE: &str = "DELETE FROM accounts WHERE id = ?1";

pub const TIERS: &str =
    "SELECT account_id, min_balance, rate FROM interest_tiers ORDER BY min_balance";

pub const DELETE_TIERS: &str = "DELETE FROM interest_tiers WHERE account_id = ?1";

pub const INSERT_TIER: &str =
    "INSERT INTO interest_tiers (account_id, min_balance, rate) VALUES (?1, ?2, ?3)";
