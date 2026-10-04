DROP INDEX idx_transactions_account_date;
DROP INDEX idx_transactions_date;

CREATE INDEX idx_transactions_account_date ON transactions (account_id, date, amount, confirmed);
CREATE INDEX idx_transactions_date ON transactions (date, kind, account_id, category_id, amount);
