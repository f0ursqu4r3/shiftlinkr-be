-- Core schema: users, companies, and user-company relationships
-- This migration creates the foundational tables for the ShiftLinkr application
-- Users table
CREATE TABLE
    users (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        email VARCHAR(255) NOT NULL UNIQUE,
        password_hash VARCHAR(255) NOT NULL,
        name VARCHAR(255) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Companies table
CREATE TABLE
    companies (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        name VARCHAR(255) NOT NULL,
        description TEXT,
        website VARCHAR(255),
        phone VARCHAR(50),
        email VARCHAR(255),
        address TEXT,
        logo_url VARCHAR(500),
        timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- User-company relationships with roles and PTO information
CREATE TABLE
    user_company (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        role VARCHAR(50) NOT NULL DEFAULT 'employee',
        is_primary BOOLEAN NOT NULL DEFAULT FALSE,
        hire_date DATE,
        pto_balance_hours INTEGER NOT NULL DEFAULT 0,
        sick_balance_hours INTEGER NOT NULL DEFAULT 0,
        personal_balance_hours INTEGER NOT NULL DEFAULT 0,
        pto_accrual_rate DECIMAL(5, 2) NOT NULL DEFAULT 0.0,
        last_accrual_date DATE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (user_id, company_id)
    );

-- Indexes for performance
CREATE INDEX idx_users_email ON users (email);

CREATE INDEX idx_companies_name ON companies (name);

CREATE INDEX idx_user_company_user_id ON user_company (user_id);

CREATE INDEX idx_user_company_company_id ON user_company (company_id);

CREATE INDEX idx_user_company_role ON user_company (role);
