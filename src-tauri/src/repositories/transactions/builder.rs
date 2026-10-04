use sqlx::{QueryBuilder, Sqlite};

use crate::models::TransactionFilter;

pub fn push_filter(builder: &mut QueryBuilder<'_, Sqlite>, filter: &TransactionFilter) {
    builder.push(" WHERE 1 = 1");
    and(
        builder,
        "t.account_id = ",
        filter.account_id.map(|id| id.hyphenated().to_string()),
    );
    and(builder, "t.kind = ", filter.kind.map(|kind| kind.as_str()));
    and(builder, "t.date >= ", filter.date_from);
    and(builder, "t.date <= ", filter.date_to);
    and(builder, "abs(t.amount) >= ", filter.min_amount);
    and(builder, "abs(t.amount) <= ", filter.max_amount);
    and(builder, "t.confirmed = ", filter.confirmed);
    push_category(builder, filter);
    push_search(builder, filter);
}

fn and<'a, T>(builder: &mut QueryBuilder<'a, Sqlite>, clause: &str, value: Option<T>)
where
    T: 'a + sqlx::Encode<'a, Sqlite> + sqlx::Type<Sqlite>,
{
    if let Some(value) = value {
        builder.push(" AND ").push(clause).push_bind(value);
    }
}

fn push_category(builder: &mut QueryBuilder<'_, Sqlite>, filter: &TransactionFilter) {
    let Some(id) = filter.category_id else { return };
    let id = id.hyphenated().to_string();
    builder
        .push(" AND (t.category_id = ")
        .push_bind(id.clone())
        .push(" OR t.category_id IN (SELECT id FROM categories WHERE parent_id = ")
        .push_bind(id)
        .push("))");
}

fn push_search(builder: &mut QueryBuilder<'_, Sqlite>, filter: &TransactionFilter) {
    let Some(search) = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    let pattern = format!("%{}%", search.replace(['%', '_'], ""));
    builder
        .push(" AND (t.description LIKE ")
        .push_bind(pattern.clone())
        .push(" OR t.notes LIKE ")
        .push_bind(pattern)
        .push(")");
}
