CREATE TABLE accounts_new (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('bank', 'savings', 'card', 'cash', 'meal', 'other')),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    initial_balance INTEGER NOT NULL DEFAULT 0,
    color TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    icon TEXT,
    interest_period_months INTEGER CHECK (interest_period_months IS NULL OR interest_period_months > 0)
);

INSERT INTO accounts_new (id, name, kind, currency, initial_balance, color, archived_at, created_at, updated_at, icon)
SELECT id, name, kind, currency, initial_balance, color, archived_at, created_at, updated_at, icon FROM accounts;

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;
CREATE INDEX idx_accounts_archived_at ON accounts (archived_at);

CREATE TABLE interest_tiers (
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    min_balance INTEGER NOT NULL CHECK (min_balance >= 0),
    rate REAL NOT NULL CHECK (rate >= 0),
    PRIMARY KEY (account_id, min_balance)
);

ALTER TABLE transactions ADD COLUMN one_off INTEGER NOT NULL DEFAULT 0 CHECK (one_off IN (0, 1));

CREATE TABLE dismissed_alerts (
    key TEXT PRIMARY KEY NOT NULL,
    dismissed_at TEXT NOT NULL
);
