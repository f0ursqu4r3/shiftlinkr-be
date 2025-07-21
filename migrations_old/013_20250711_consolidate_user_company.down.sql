-- Rollback: Split user_company back into company_employees and user_company
-- Recreate company_employees table
CREATE TABLE IF NOT EXISTS company_employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    company_id INTEGER NOT NULL,
    role TEXT NOT NULL DEFAULT 'employee',
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    hired_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    UNIQUE(user_id, company_id)
);

-- Migrate data back to company_employees
INSERT INTO
    company_employees (
        user_id,
        company_id,
        role,
        is_primary,
        hired_at,
        created_at,
        updated_at
    )
SELECT
    user_id,
    company_id,
    role,
    is_primary,
    hire_date,
    created_at,
    updated_at
FROM
    user_company;

-- Recreate company_employees indexes
CREATE INDEX IF NOT EXISTS idx_company_employees_user_id ON company_employees(user_id);

CREATE INDEX IF NOT EXISTS idx_company_employees_company_id ON company_employees(company_id);

CREATE INDEX IF NOT EXISTS idx_company_employees_role ON company_employees(role);

CREATE INDEX IF NOT EXISTS idx_company_employees_is_primary ON company_employees(is_primary);

-- Remove the consolidated columns from user_company
-- Note: SQLite doesn't support DROP COLUMN, so we need to recreate the table
CREATE TABLE user_company_new (
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

-- Copy data back (excluding role and is_primary)
INSERT INTO
    user_company_new (
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
FROM
    user_company;

-- Replace the table
DROP TABLE user_company;

ALTER TABLE
    user_company_new RENAME TO user_company;

-- Recreate user_company indexes (without role and is_primary indexes)
CREATE INDEX IF NOT EXISTS idx_user_company_user_id ON user_company(user_id);

CREATE INDEX IF NOT EXISTS idx_user_company_company_id ON user_company(company_id);
