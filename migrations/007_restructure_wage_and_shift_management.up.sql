-- Restructure wage management and improve shift schema
-- Moves hourly_rate from shifts to user_company, adds wage history tracking,
-- and improves shift table structure for better multi-tenancy and capacity management
-- 1. Add hourly_rate to user_company table (where it belongs)
ALTER TABLE user_company
ADD COLUMN IF NOT EXISTS hourly_rate DECIMAL(10, 2),
ADD COLUMN IF NOT EXISTS overtime_rate_multiplier DECIMAL(3, 2) DEFAULT 1.5;

-- 2. Create wage history table for tracking rate changes
CREATE TABLE
    IF NOT EXISTS wage_history (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        hourly_rate DECIMAL(10, 2) NOT NULL,
        overtime_rate_multiplier DECIMAL(3, 2) DEFAULT 1.5,
        effective_date DATE NOT NULL,
        end_date DATE,
        changed_by UUID REFERENCES users (id) ON DELETE SET NULL,
        change_reason TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- 3. Add missing company_id to shifts for better isolation
ALTER TABLE shifts
ADD COLUMN IF NOT EXISTS company_id UUID REFERENCES companies (id) ON DELETE CASCADE;

-- 4. Populate company_id from location relationship
UPDATE shifts s
SET
    company_id = l.company_id
FROM
    locations l
WHERE
    s.location_id = l.id
    AND s.company_id IS NULL;

-- Make company_id NOT NULL after population
ALTER TABLE shifts
ALTER COLUMN company_id
SET
    NOT NULL;

-- 5. Add missing shift capacity fields
ALTER TABLE shifts
ADD COLUMN IF NOT EXISTS min_duration_minutes INTEGER,
ADD COLUMN IF NOT EXISTS max_duration_minutes INTEGER,
ADD COLUMN IF NOT EXISTS max_people INTEGER DEFAULT 1;

-- 6. Migrate existing hourly_rate data to user_company
-- This assumes one rate per user per company (takes the most recent shift rate)
UPDATE user_company uc
SET
    hourly_rate = subquery.latest_rate
FROM
    (
        SELECT DISTINCT
            ON (s.assigned_user_id, l.company_id) s.assigned_user_id AS user_id,
            l.company_id,
            s.hourly_rate AS latest_rate
        FROM
            shifts s
            JOIN locations l ON s.location_id = l.id
        WHERE
            s.assigned_user_id IS NOT NULL
            AND s.hourly_rate IS NOT NULL
        ORDER BY
            s.assigned_user_id,
            l.company_id,
            s.created_at DESC
    ) subquery
WHERE
    uc.user_id = subquery.user_id
    AND uc.company_id = subquery.company_id
    AND uc.hourly_rate IS NULL;

-- 7. Remove the redundant assigned_user_id from shifts
-- (using shift_proposal_assignments table instead)
ALTER TABLE shifts
DROP COLUMN IF EXISTS assigned_user_id;

-- 8. Remove hourly_rate from shifts table
ALTER TABLE shifts
DROP COLUMN IF EXISTS hourly_rate;

-- 9. Add useful indexes
CREATE INDEX IF NOT EXISTS idx_shifts_company_id ON shifts (company_id);

CREATE INDEX IF NOT EXISTS idx_wage_history_user_company ON wage_history (user_id, company_id);

CREATE INDEX IF NOT EXISTS idx_wage_history_effective_date ON wage_history (effective_date);

-- 10. Add check constraints for data integrity
ALTER TABLE shifts ADD CONSTRAINT check_shift_times CHECK (end_time > start_time);

ALTER TABLE shifts ADD CONSTRAINT check_max_people CHECK (max_people > 0);

ALTER TABLE wage_history ADD CONSTRAINT check_wage_dates CHECK (
    end_date IS NULL
    OR end_date > effective_date
);
