CREATE TABLE accounts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('bank', 'savings', 'card', 'cash', 'other')),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    initial_balance INTEGER NOT NULL DEFAULT 0,
    color TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_accounts_archived_at ON accounts (archived_at);
