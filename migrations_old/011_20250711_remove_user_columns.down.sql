-- Restore the old users table structure with PTO balance and role columns
-- This is for rollback purposes
CREATE TABLE users_new (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'employee',
    pto_balance_hours INTEGER NOT NULL DEFAULT 0,
    sick_balance_hours INTEGER NOT NULL DEFAULT 0,
    personal_balance_hours INTEGER NOT NULL DEFAULT 0,
    pto_accrual_rate REAL NOT NULL DEFAULT 0.0,
    hire_date DATE,
    last_accrual_date DATE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Copy data from current table
INSERT INTO
    users_new (
        id,
        email,
        password_hash,
        name,
        hire_date,
        created_at,
        updated_at
    )
SELECT
    id,
    email,
    password_hash,
    name,
    hire_date,
    created_at,
    updated_at
FROM
    users;

-- Restore role and PTO balance data from company_employees and user_company tables
UPDATE
    users_new
SET
    role = (
        SELECT
            ce.role
        FROM
            company_employees ce
        WHERE
            ce.user_id = users_new.id
        LIMIT
            1
    );

UPDATE
    users_new
SET
    pto_balance_hours = (
        SELECT
            uc.pto_balance_hours
        FROM
            user_company uc
        WHERE
            uc.user_id = users_new.id
        LIMIT
            1
    ), sick_balance_hours = (
        SELECT
            uc.sick_balance_hours
        FROM
            user_company uc
        WHERE
            uc.user_id = users_new.id
        LIMIT
            1
    ), personal_balance_hours = (
        SELECT
            uc.personal_balance_hours
        FROM
            user_company uc
        WHERE
            uc.user_id = users_new.id
        LIMIT
            1
    ), pto_accrual_rate = (
        SELECT
            uc.pto_accrual_rate
        FROM
            user_company uc
        WHERE
            uc.user_id = users_new.id
        LIMIT
            1
    ), last_accrual_date = (
        SELECT
            uc.last_accrual_date
        FROM
            user_company uc
        WHERE
            uc.user_id = users_new.id
        LIMIT
            1
    )
WHERE
    EXISTS (
        SELECT
            1
        FROM
            user_company uc
        WHERE
            uc.user_id = users_new.id
    );

-- Drop current table and rename
DROP TABLE users;

ALTER TABLE
    users_new RENAME TO users;

-- Recreate index
CREATE INDEX idx_users_email ON users(email);
