ALTER TABLE accounts ADD COLUMN icon TEXT;

CREATE TABLE categories (
    id TEXT PRIMARY KEY NOT NULL,
    parent_id TEXT REFERENCES categories (id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('income', 'outcome', 'both')),
    icon TEXT,
    color TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (parent_id IS NULL OR parent_id <> id)
);

CREATE INDEX idx_categories_parent ON categories (parent_id);

CREATE TABLE transfers (
    id TEXT PRIMARY KEY NOT NULL,
    from_account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    to_account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    amount INTEGER NOT NULL CHECK (amount > 0),
    date TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (from_account_id <> to_account_id)
);

CREATE TABLE credits (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    institution TEXT,
    principal INTEGER NOT NULL CHECK (principal > 0),
    opening_balance INTEGER NOT NULL CHECK (opening_balance >= 0),
    annual_rate REAL NOT NULL CHECK (annual_rate >= 0),
    installment INTEGER NOT NULL CHECK (installment > 0),
    unit TEXT NOT NULL CHECK (unit IN ('day', 'week', 'month', 'year')),
    interval INTEGER NOT NULL CHECK (interval > 0),
    start_date TEXT NOT NULL,
    end_date TEXT,
    installments INTEGER CHECK (installments IS NULL OR installments > 0),
    account_id TEXT REFERENCES accounts (id) ON DELETE RESTRICT,
    archived_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE recurrences (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    category_id TEXT REFERENCES categories (id) ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK (kind IN ('income', 'outcome')),
    amount INTEGER NOT NULL CHECK (amount > 0),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    start_date TEXT NOT NULL,
    end_date TEXT,
    unit TEXT NOT NULL CHECK (unit IN ('day', 'week', 'month', 'year')),
    interval INTEGER NOT NULL CHECK (interval > 0),
    paused_at TEXT,
    credit_id TEXT REFERENCES credits (id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (end_date IS NULL OR end_date >= start_date)
);

CREATE INDEX idx_recurrences_account ON recurrences (account_id);

CREATE TABLE transactions (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    category_id TEXT REFERENCES categories (id) ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK (kind IN ('income', 'outcome', 'transfer')),
    amount INTEGER NOT NULL,
    date TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    notes TEXT,
    confirmed INTEGER NOT NULL DEFAULT 0 CHECK (confirmed IN (0, 1)),
    transfer_id TEXT REFERENCES transfers (id) ON DELETE CASCADE,
    recurrence_id TEXT REFERENCES recurrences (id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (
        (kind = 'income' AND amount > 0 AND transfer_id IS NULL)
        OR (kind = 'outcome' AND amount < 0 AND transfer_id IS NULL)
        OR (kind = 'transfer' AND amount <> 0 AND transfer_id IS NOT NULL AND category_id IS NULL)
    )
);

CREATE INDEX idx_transactions_account_date ON transactions (account_id, date);
CREATE INDEX idx_transactions_date ON transactions (date);
CREATE INDEX idx_transactions_category ON transactions (category_id);
CREATE INDEX idx_transactions_transfer ON transactions (transfer_id);
CREATE INDEX idx_transactions_recurrence ON transactions (recurrence_id);

CREATE TABLE recurrence_occurrences (
    recurrence_id TEXT NOT NULL REFERENCES recurrences (id) ON DELETE CASCADE,
    occurrence_date TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'skipped', 'done')),
    amount INTEGER CHECK (amount IS NULL OR amount > 0),
    date TEXT,
    transaction_id TEXT REFERENCES transactions (id) ON DELETE SET NULL,
    PRIMARY KEY (recurrence_id, occurrence_date)
);

CREATE TABLE credit_payments (
    id TEXT PRIMARY KEY NOT NULL,
    credit_id TEXT NOT NULL REFERENCES credits (id) ON DELETE CASCADE,
    transaction_id TEXT REFERENCES transactions (id) ON DELETE SET NULL,
    date TEXT NOT NULL,
    principal INTEGER NOT NULL CHECK (principal >= 0),
    interest INTEGER NOT NULL CHECK (interest >= 0),
    created_at TEXT NOT NULL
);

CREATE INDEX idx_credit_payments_credit ON credit_payments (credit_id, date);

CREATE TABLE goals (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    target_amount INTEGER NOT NULL CHECK (target_amount > 0),
    target_date TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    category_id TEXT REFERENCES categories (id) ON DELETE SET NULL,
    kind TEXT NOT NULL CHECK (kind IN ('income', 'outcome')),
    amount INTEGER CHECK (amount IS NULL OR amount > 0),
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE reconciliations (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    date TEXT NOT NULL,
    statement_balance INTEGER NOT NULL,
    calculated_balance INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_reconciliations_account ON reconciliations (account_id, date);

CREATE TABLE saved_filters (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    filter TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE settings (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id = 1),
    data TEXT NOT NULL
);

INSERT INTO categories (id, parent_id, name, kind, color, created_at, updated_at) VALUES
    ('00000000-0000-7000-8000-000000000001', NULL, 'Rendimentos', 'income', '#1baf7a', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000002', '00000000-0000-7000-8000-000000000001', 'Salário', 'income', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000003', '00000000-0000-7000-8000-000000000001', 'Juros', 'income', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000004', '00000000-0000-7000-8000-000000000001', 'Outros rendimentos', 'income', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000010', NULL, 'Alimentação', 'outcome', '#eb6834', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000011', '00000000-0000-7000-8000-000000000010', 'Supermercado', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000012', '00000000-0000-7000-8000-000000000010', 'Restaurantes', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000020', NULL, 'Casa', 'outcome', '#2a78d6', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000021', '00000000-0000-7000-8000-000000000020', 'Renda', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000022', '00000000-0000-7000-8000-000000000020', 'Eletricidade', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000023', '00000000-0000-7000-8000-000000000020', 'Água', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000024', '00000000-0000-7000-8000-000000000020', 'Internet', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000030', NULL, 'Transportes', 'outcome', '#eda100', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000031', '00000000-0000-7000-8000-000000000030', 'Combustível', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000032', '00000000-0000-7000-8000-000000000030', 'Manutenção', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000033', '00000000-0000-7000-8000-000000000030', 'Transportes públicos', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000040', NULL, 'Saúde', 'outcome', '#e87ba4', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000050', NULL, 'Lazer', 'outcome', '#4a3aa7', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000051', '00000000-0000-7000-8000-000000000050', 'Subscrições', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000052', '00000000-0000-7000-8000-000000000050', 'Viagens', 'outcome', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000060', NULL, 'Créditos', 'outcome', '#e34948', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('00000000-0000-7000-8000-000000000070', NULL, 'Outros', 'both', '#008300', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
