ALTER TABLE accounts ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

UPDATE accounts
SET position = (SELECT COUNT(*) FROM accounts other WHERE other.name COLLATE NOCASE < accounts.name COLLATE NOCASE);
