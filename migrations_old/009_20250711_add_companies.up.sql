-- Create companies table
CREATE TABLE IF NOT EXISTS companies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    website TEXT,
    phone TEXT,
    email TEXT,
    address TEXT,
    logo_url TEXT,
    timezone TEXT DEFAULT 'UTC',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add company_id to locations table
ALTER TABLE
    locations
ADD
    COLUMN company_id INTEGER;

-- Create company_employees junction table for user-company relationships
CREATE TABLE IF NOT EXISTS company_employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    company_id INTEGER NOT NULL,
    role TEXT NOT NULL DEFAULT 'employee',
    -- employee, manager, admin
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    -- user's primary company
    hired_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    UNIQUE(user_id, company_id)
);

-- Add foreign key constraint to locations after adding company_id
CREATE INDEX IF NOT EXISTS idx_locations_company_id ON locations(company_id);

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_company_employees_user_id ON company_employees(user_id);

CREATE INDEX IF NOT EXISTS idx_company_employees_company_id ON company_employees(company_id);

CREATE INDEX IF NOT EXISTS idx_company_employees_role ON company_employees(role);

CREATE INDEX IF NOT EXISTS idx_company_employees_is_primary ON company_employees(is_primary);

-- Create a default company for existing data
INSERT INTO
    companies (name, description, created_at, updated_at)
VALUES
    (
        'Default Company',
        'Auto-created company for existing data',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    );

-- Update existing locations to belong to the default company
UPDATE
    locations
SET
    company_id = 1
WHERE
    company_id IS NULL;

-- Add foreign key constraint now that all locations have company_id
-- Note: SQLite doesn't support adding foreign key constraints to existing tables easily
-- We'll handle this constraint at the application level for now
-- Migrate existing users to be employees of the default company
INSERT INTO
    company_employees (
        user_id,
        company_id,
        role,
        is_primary,
        created_at,
        updated_at
    )
SELECT
    id as user_id,
    1 as company_id,
    role,
    TRUE as is_primary,
    CURRENT_TIMESTAMP as created_at,
    CURRENT_TIMESTAMP as updated_at
FROM
    users
WHERE
    id NOT IN (
        SELECT
            user_id
        FROM
            company_employees
    );
