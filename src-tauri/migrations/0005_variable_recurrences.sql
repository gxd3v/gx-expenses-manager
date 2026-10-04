ALTER TABLE recurrences ADD COLUMN variable_amount INTEGER NOT NULL DEFAULT 0 CHECK (variable_amount IN (0, 1));
