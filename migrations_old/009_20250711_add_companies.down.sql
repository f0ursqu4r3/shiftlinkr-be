-- Drop foreign key constraints and indexes
DROP INDEX IF EXISTS idx_company_employees_is_primary;

DROP INDEX IF EXISTS idx_company_employees_role;

DROP INDEX IF EXISTS idx_company_employees_company_id;

DROP INDEX IF EXISTS idx_company_employees_user_id;

DROP INDEX IF EXISTS idx_locations_company_id;

-- Drop tables (order matters due to foreign keys)
DROP TABLE IF EXISTS company_employees;

-- Remove company_id column from locations
-- Note: SQLite doesn't support DROP COLUMN directly, so we'll recreate the table
CREATE TABLE locations_backup AS
SELECT
    id,
    name,
    address,
    phone,
    email,
    created_at,
    updated_at
FROM
    locations;

DROP TABLE locations;

CREATE TABLE locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    address TEXT,
    phone TEXT,
    email TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO
    locations
SELECT
    *
FROM
    locations_backup;

DROP TABLE locations_backup;

-- Drop companies table
DROP TABLE IF EXISTS companies;
