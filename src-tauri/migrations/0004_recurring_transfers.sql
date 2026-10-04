ALTER TABLE recurrences ADD COLUMN to_account_id TEXT REFERENCES accounts (id) ON DELETE RESTRICT;
