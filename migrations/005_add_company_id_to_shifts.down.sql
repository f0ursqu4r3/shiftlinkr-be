-- Remove company_id from shifts table
-- Drop indexes first
DROP INDEX IF EXISTS idx_shifts_company_date_range;

DROP INDEX IF EXISTS idx_shifts_company_status;

DROP INDEX IF EXISTS idx_shifts_company_location;

DROP INDEX IF EXISTS idx_shifts_company_id;

-- Remove the company_id column
ALTER TABLE shifts
DROP COLUMN company_id;
