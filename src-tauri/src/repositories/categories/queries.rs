macro_rules! select {
    () => {
        "SELECT c.id, c.parent_id, c.name, c.kind, c.icon, c.color, c.archived_at, c.created_at, c.updated_at, \
         (SELECT COUNT(*) FROM transactions t WHERE t.category_id = c.id) AS transaction_count \
         FROM categories c "
    };
}

pub const LIST: &str = concat!(
    select!(),
    "WHERE ?1 OR c.archived_at IS NULL ORDER BY c.name COLLATE NOCASE"
);

pub const GET: &str = concat!(select!(), "WHERE c.id = ?1");

pub const INSERT: &str = "INSERT INTO categories (id, parent_id, name, kind, icon, color, created_at, updated_at) \
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)";

pub const UPDATE: &str = "UPDATE categories \
     SET parent_id = ?2, name = ?3, kind = ?4, icon = ?5, color = ?6, updated_at = ?7 \
     WHERE id = ?1";

pub const SET_ARCHIVED: &str =
    "UPDATE categories SET archived_at = ?2, updated_at = ?3 WHERE id = ?1";

pub const HAS_CHILDREN: &str = "SELECT EXISTS (SELECT 1 FROM categories WHERE parent_id = ?1)";

pub const IN_USE: &str = "SELECT EXISTS (SELECT 1 FROM transactions WHERE category_id = ?1) \
     OR EXISTS (SELECT 1 FROM recurrences WHERE category_id = ?1)";

pub const REASSIGN: [&str; 3] = [
    "UPDATE transactions SET category_id = ?2 WHERE category_id = ?1",
    "UPDATE recurrences SET category_id = ?2 WHERE category_id = ?1",
    "UPDATE templates SET category_id = ?2 WHERE category_id = ?1",
];

pub const DELETE: &str = "DELETE FROM categories WHERE id = ?1";
