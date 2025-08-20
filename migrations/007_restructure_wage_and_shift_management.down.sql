-- Revert structural changes to shifts and wage management
-- Remove constraints
ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS check_shift_times;

ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS check_max_people;

ALTER TABLE wage_history
DROP CONSTRAINT IF EXISTS check_wage_dates;

-- Remove indexes
DROP INDEX IF EXISTS idx_shifts_company_id;

DROP INDEX IF EXISTS idx_wage_history_user_company;

DROP INDEX IF EXISTS idx_wage_history_effective_date;

-- Re-add assigned_user_id to shifts
ALTER TABLE shifts
ADD COLUMN IF NOT EXISTS assigned_user_id UUID REFERENCES users (id) ON DELETE SET NULL;

-- Re-add hourly_rate to shifts
ALTER TABLE shifts
ADD COLUMN IF NOT EXISTS hourly_rate DECIMAL(10, 2);

-- Remove company_id from shifts
ALTER TABLE shifts
DROP COLUMN IF EXISTS company_id;

-- Remove shift capacity fields
ALTER TABLE shifts
DROP COLUMN IF EXISTS min_duration_minutes,
DROP COLUMN IF EXISTS max_duration_minutes,
DROP COLUMN IF EXISTS max_people;

-- Remove wage columns from user_company
ALTER TABLE user_company
DROP COLUMN IF EXISTS hourly_rate,
DROP COLUMN IF EXISTS overtime_rate_multiplier;

-- Drop wage history table
DROP TABLE IF EXISTS wage_history;
