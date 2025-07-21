-- Organizational structure: locations, teams, and shifts
-- Locations within companies
CREATE TABLE locations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    address TEXT,
    phone VARCHAR(50),
    email VARCHAR(255),
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Teams within locations
CREATE TABLE teams (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Team memberships
CREATE TABLE team_members (
    id SERIAL PRIMARY KEY,
    team_id INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(team_id, user_id)
);

-- Shifts (scheduled work periods)
CREATE TABLE shifts (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    team_id INTEGER REFERENCES teams(id) ON DELETE
    SET
        NULL,
        assigned_user_id UUID REFERENCES users(id) ON DELETE
    SET
        NULL,
        start_time TIMESTAMPTZ NOT NULL,
        end_time TIMESTAMPTZ NOT NULL,
        hourly_rate DECIMAL(10, 2),
        STATUS VARCHAR(50) NOT NULL DEFAULT 'open',
        -- 'open', 'assigned', 'completed', 'cancelled'
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Time off requests
CREATE TABLE time_off_requests (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    reason TEXT,
    request_type VARCHAR(50) NOT NULL,
    -- 'Vacation', 'Sick', 'Personal', etc.
    STATUS VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- 'pending', 'approved', 'denied', 'cancelled'
    approved_by UUID REFERENCES users(id) ON DELETE
    SET
        NULL,
        approval_notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Shift swaps
CREATE TABLE shift_swaps (
    id SERIAL PRIMARY KEY,
    requesting_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    original_shift_id INTEGER NOT NULL REFERENCES shifts(id) ON DELETE CASCADE,
    target_user_id UUID REFERENCES users(id) ON DELETE
    SET
        NULL,
        target_shift_id INTEGER REFERENCES shifts(id) ON DELETE
    SET
        NULL,
        notes TEXT,
        swap_type VARCHAR(50) NOT NULL,
        -- 'open', 'targeted'
        STATUS VARCHAR(50) NOT NULL DEFAULT 'open',
        -- 'open', 'pending', 'approved', 'denied', 'completed', 'cancelled'
        approved_by UUID REFERENCES users(id) ON DELETE
    SET
        NULL,
        approval_notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Shift claims (for open shifts)
CREATE TABLE shift_claims (
    id SERIAL PRIMARY KEY,
    shift_id INTEGER NOT NULL REFERENCES shifts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    STATUS VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- 'pending', 'approved', 'rejected', 'cancelled'
    approved_by UUID REFERENCES users(id) ON DELETE
    SET
        NULL,
        approval_notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE(shift_id, user_id)
);

-- Add foreign key reference for team_id in invite_tokens
ALTER TABLE
    invite_tokens
ADD
    CONSTRAINT fk_invite_tokens_team_id FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE
SET
    NULL;

-- Add foreign key reference for related_time_off_id in pto_balance_history
ALTER TABLE
    pto_balance_history
ADD
    CONSTRAINT fk_pto_balance_history_time_off_id FOREIGN KEY (related_time_off_id) REFERENCES time_off_requests(id) ON DELETE
SET
    NULL;

-- Indexes for performance
CREATE INDEX idx_locations_company_id ON locations(company_id);

CREATE INDEX idx_teams_location_id ON teams(location_id);

CREATE INDEX idx_team_members_team_id ON team_members(team_id);

CREATE INDEX idx_team_members_user_id ON team_members(user_id);

CREATE INDEX idx_shifts_location_id ON shifts(location_id);

CREATE INDEX idx_shifts_team_id ON shifts(team_id);

CREATE INDEX idx_shifts_assigned_user_id ON shifts(assigned_user_id);

CREATE INDEX idx_shifts_start_time ON shifts(start_time);

CREATE INDEX idx_shifts_status ON shifts(STATUS);

CREATE INDEX idx_time_off_requests_user_id ON time_off_requests(user_id);

CREATE INDEX idx_time_off_requests_company_id ON time_off_requests(company_id);

CREATE INDEX idx_time_off_requests_status ON time_off_requests(STATUS);

CREATE INDEX idx_time_off_requests_dates ON time_off_requests(start_date, end_date);

CREATE INDEX idx_shift_swaps_requesting_user_id ON shift_swaps(requesting_user_id);

CREATE INDEX idx_shift_swaps_original_shift_id ON shift_swaps(original_shift_id);

CREATE INDEX idx_shift_swaps_target_user_id ON shift_swaps(target_user_id);

CREATE INDEX idx_shift_swaps_status ON shift_swaps(STATUS);

CREATE INDEX idx_shift_claims_shift_id ON shift_claims(shift_id);

CREATE INDEX idx_shift_claims_user_id ON shift_claims(user_id);

CREATE INDEX idx_shift_claims_status ON shift_claims(STATUS);
