CREATE TABLE accounts_new (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('bank', 'savings', 'card', 'credit_card', 'cash', 'meal', 'other')),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    initial_balance INTEGER NOT NULL DEFAULT 0,
    color TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    icon TEXT,
    interest_period_months INTEGER CHECK (interest_period_months IS NULL OR interest_period_months > 0),
    overdraft_limit INTEGER NOT NULL DEFAULT 0 CHECK (overdraft_limit >= 0)
);

INSERT INTO accounts_new (id, name, kind, currency, initial_balance, color, archived_at, created_at, updated_at, icon, interest_period_months)
SELECT id, name, kind, currency, initial_balance, color, archived_at, created_at, updated_at, icon, interest_period_months FROM accounts;

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;
CREATE INDEX idx_accounts_archived_at ON accounts (archived_at);

CREATE TABLE wishlist_items (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    amount INTEGER NOT NULL CHECK (amount > 0),
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 1 AND 3),
    notes TEXT,
    purchased_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_wishlist_items_account ON wishlist_items (account_id);
