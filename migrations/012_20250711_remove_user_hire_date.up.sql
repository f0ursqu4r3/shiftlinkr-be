-- Remove hire_date from users table since it's now only in company_employees
-- SQLite doesn't support DROP COLUMN, so we need to recreate the table
CREATE TABLE users_new (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Copy data from old table to new table (excluding hire_date)
INSERT INTO
    users_new (
        id,
        email,
        password_hash,
        name,
        created_at,
        updated_at
    )
SELECT
    id,
    email,
    password_hash,
    name,
    created_at,
    updated_at
FROM
    users;

-- Drop old table and rename new table
DROP TABLE users;

ALTER TABLE
    users_new RENAME TO users;

-- Recreate the index
CREATE INDEX idx_users_email ON users(email);
