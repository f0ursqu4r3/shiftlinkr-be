-- Add PTO balance fields to users table
ALTER TABLE
    users
ADD
    COLUMN pto_balance_hours INTEGER DEFAULT 0;

ALTER TABLE
    users
ADD
    COLUMN sick_balance_hours INTEGER DEFAULT 0;

ALTER TABLE
    users
ADD
    COLUMN personal_balance_hours INTEGER DEFAULT 0;

ALTER TABLE
    users
ADD
    COLUMN pto_accrual_rate REAL DEFAULT 0.0;

ALTER TABLE
    users
ADD
    COLUMN hire_date DATE;

ALTER TABLE
    users
ADD
    COLUMN last_accrual_date DATE;

-- Create PTO balance history table for tracking accruals and usage
CREATE TABLE IF NOT EXISTS pto_balance_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    balance_type TEXT NOT NULL,
    -- 'pto', 'sick', 'personal'
    change_type TEXT NOT NULL,
    -- 'accrual', 'usage', 'adjustment'
    hours_changed INTEGER NOT NULL,
    previous_balance INTEGER NOT NULL,
    new_balance INTEGER NOT NULL,
    description TEXT,
    related_time_off_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (related_time_off_id) REFERENCES time_off_requests(id) ON DELETE
    SET
        NULL
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_pto_balance_history_user_id ON pto_balance_history(user_id);

CREATE INDEX IF NOT EXISTS idx_pto_balance_history_balance_type ON pto_balance_history(balance_type);

CREATE INDEX IF NOT EXISTS idx_pto_balance_history_created_at ON pto_balance_history(created_at);
