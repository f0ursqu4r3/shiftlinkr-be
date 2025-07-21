-- Create user_company table for company-specific PTO/sick balances
CREATE TABLE IF NOT EXISTS user_company (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    company_id INTEGER NOT NULL,
    pto_balance_hours INTEGER NOT NULL DEFAULT 0,
    sick_balance_hours INTEGER NOT NULL DEFAULT 0,
    personal_balance_hours INTEGER NOT NULL DEFAULT 0,
    pto_accrual_rate REAL NOT NULL DEFAULT 0.0,
    hire_date DATETIME,
    last_accrual_date DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    UNIQUE(user_id, company_id)
);

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_company_user_id ON user_company(user_id);

CREATE INDEX IF NOT EXISTS idx_user_company_company_id ON user_company(company_id);

-- Migrate existing user balances to the new table
INSERT INTO
    user_company (
        user_id,
        company_id,
        pto_balance_hours,
        sick_balance_hours,
        personal_balance_hours,
        pto_accrual_rate,
        hire_date,
        last_accrual_date,
        created_at,
        updated_at
    )
SELECT
    u.id as user_id,
    ce.company_id,
    u.pto_balance_hours,
    u.sick_balance_hours,
    u.personal_balance_hours,
    u.pto_accrual_rate,
    u.hire_date,
    u.last_accrual_date,
    CURRENT_TIMESTAMP as created_at,
    CURRENT_TIMESTAMP as updated_at
FROM
    users u
    JOIN company_employees ce ON u.id = ce.user_id
WHERE
    NOT EXISTS (
        SELECT
            1
        FROM
            user_company ucb
        WHERE
            ucb.user_id = u.id
            AND ucb.company_id = ce.company_id
    );
