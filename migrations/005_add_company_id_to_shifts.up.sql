-- Add company_id to shifts table for multi-tenant data isolation
-- Add the company_id column as nullable first
ALTER TABLE shifts
ADD COLUMN company_id UUID REFERENCES companies (id) ON DELETE CASCADE;

-- Update existing records to set company_id based on the location's company
UPDATE shifts
SET
    company_id = (
        SELECT
            l.company_id
        FROM
            locations l
        WHERE
            l.id = shifts.location_id
        LIMIT
            1
    );

-- Make the column NOT NULL after populating existing data
ALTER TABLE shifts
ALTER COLUMN company_id
SET
    NOT NULL;

-- Add index for performance
CREATE INDEX idx_shifts_company_id ON shifts (company_id);

-- Add composite index for common query patterns
CREATE INDEX idx_shifts_company_location ON shifts (company_id, location_id);

CREATE INDEX idx_shifts_company_status ON shifts (company_id, status);

CREATE INDEX idx_shifts_company_date_range ON shifts (company_id, start_time, end_time);
