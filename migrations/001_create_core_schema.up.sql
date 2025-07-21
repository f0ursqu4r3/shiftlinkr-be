-- Core users and authentication system
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table - basic user information
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Password reset tokens
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Companies
CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    website VARCHAR(255),
    phone VARCHAR(50),
    email VARCHAR(255),
    address TEXT,
    logo_url VARCHAR(500),
    timezone VARCHAR(100) NOT NULL DEFAULT 'UTC',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User-Company relationship with role and PTO information
CREATE TABLE user_company (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'employee',
    -- 'employee', 'manager', 'admin'
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    hire_date DATE,
    -- PTO/Time off fields
    pto_balance_hours INTEGER NOT NULL DEFAULT 0,
    sick_balance_hours INTEGER NOT NULL DEFAULT 0,
    personal_balance_hours INTEGER NOT NULL DEFAULT 0,
    pto_accrual_rate DECIMAL(5, 2) NOT NULL DEFAULT 0.0,
    -- hours per pay period
    last_accrual_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, company_id)
);

-- Invite tokens for user invitations
CREATE TABLE invite_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE,
    inviter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'employee',
    team_id INTEGER,
    -- Will reference teams table (created later)
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- PTO balance history for tracking changes
CREATE TABLE pto_balance_history (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    balance_type VARCHAR(20) NOT NULL,
    -- 'pto', 'sick', 'personal'
    change_type VARCHAR(20) NOT NULL,
    -- 'accrual', 'usage', 'adjustment'
    hours_changed INTEGER NOT NULL,
    previous_balance INTEGER NOT NULL,
    new_balance INTEGER NOT NULL,
    description TEXT,
    related_time_off_id INTEGER,
    -- Will reference time_off_requests table
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Company activities log
CREATE TABLE company_activities (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE
    SET
        NULL,
        -- Who performed the action
        activity_type VARCHAR(100) NOT NULL,
        description TEXT NOT NULL,
        metadata JSONB,
        -- Additional data about the activity
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_users_email ON users(email);

CREATE INDEX idx_password_reset_tokens_token ON password_reset_tokens(token);

CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);

CREATE INDEX idx_user_company_user_id ON user_company(user_id);

CREATE INDEX idx_user_company_company_id ON user_company(company_id);

CREATE INDEX idx_invite_tokens_token ON invite_tokens(token);

CREATE INDEX idx_invite_tokens_email ON invite_tokens(email);

CREATE INDEX idx_pto_balance_history_user_id ON pto_balance_history(user_id);

CREATE INDEX idx_pto_balance_history_company_id ON pto_balance_history(company_id);

CREATE INDEX idx_company_activities_company_id ON company_activities(company_id);
