-- Consolidate company_employees into user_company table
-- Add missing columns from company_employees to user_company
-- First, add the missing columns to user_company
ALTER TABLE
    user_company
ADD
    COLUMN role TEXT NOT NULL DEFAULT 'employee';

ALTER TABLE
    user_company
ADD
    COLUMN is_primary BOOLEAN NOT NULL DEFAULT FALSE;

-- Migrate role and is_primary data from company_employees to user_company
UPDATE
    user_company
SET
    role = (
        SELECT
            ce.role
        FROM
            company_employees ce
        WHERE
            ce.user_id = user_company.user_id
            AND ce.company_id = user_company.company_id
    ),
    is_primary = (
        SELECT
            ce.is_primary
        FROM
            company_employees ce
        WHERE
            ce.user_id = user_company.user_id
            AND ce.company_id = user_company.company_id
    ),
    -- Update hire_date from company_employees if it's more recent or user_company is null
    hire_date = COALESCE(
        user_company.hire_date,
        (
            SELECT
                ce.hired_at
            FROM
                company_employees ce
            WHERE
                ce.user_id = user_company.user_id
                AND ce.company_id = user_company.company_id
        )
    )
WHERE
    EXISTS (
        SELECT
            1
        FROM
            company_employees ce
        WHERE
            ce.user_id = user_company.user_id
            AND ce.company_id = user_company.company_id
    );

-- Insert any company_employees records that don't exist in user_company
-- (in case there are employees without PTO data)
INSERT INTO
    user_company (
        user_id,
        company_id,
        role,
        is_primary,
        hire_date,
        pto_balance_hours,
        sick_balance_hours,
        personal_balance_hours,
        pto_accrual_rate,
        last_accrual_date,
        created_at,
        updated_at
    )
SELECT
    ce.user_id,
    ce.company_id,
    ce.role,
    ce.is_primary,
    ce.hired_at,
    0,
    -- default PTO balance
    0,
    -- default sick balance
    0,
    -- default personal balance
    0.0,
    -- default accrual rate
    NULL,
    -- no last accrual date
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
FROM
    company_employees ce
WHERE
    NOT EXISTS (
        SELECT
            1
        FROM
            user_company uc
        WHERE
            uc.user_id = ce.user_id
            AND uc.company_id = ce.company_id
    );

-- Add indexes for the new columns
CREATE INDEX IF NOT EXISTS idx_user_company_role ON user_company(role);

CREATE INDEX IF NOT EXISTS idx_user_company_is_primary ON user_company(is_primary);

-- Drop the old company_employees table and its indexes
DROP INDEX IF EXISTS idx_company_employees_is_primary;

DROP INDEX IF EXISTS idx_company_employees_role;

DROP INDEX IF EXISTS idx_company_employees_company_id;

DROP INDEX IF EXISTS idx_company_employees_user_id;

DROP TABLE IF EXISTS company_employees;
