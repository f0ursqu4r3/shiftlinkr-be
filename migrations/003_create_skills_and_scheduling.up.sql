-- Skills system and advanced scheduling
-- Skills that can be assigned to users and required for shifts
CREATE TABLE skills (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    category VARCHAR(50) NOT NULL DEFAULT 'general',
    -- 'general', 'certification', 'equipment', 'management'
    is_certification BOOLEAN NOT NULL DEFAULT FALSE,
    expires BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User skills mapping with proficiency levels
CREATE TABLE user_skills (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    skill_id INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    proficiency_level VARCHAR(50) NOT NULL DEFAULT 'Beginner',
    -- 'Beginner', 'Intermediate', 'Advanced', 'Expert'
    certified_date DATE,
    expiration_date DATE,
    verified_by UUID REFERENCES users(id) ON DELETE
    SET
        NULL,
        -- Manager who verified
        notes TEXT,
        is_active BOOLEAN NOT NULL DEFAULT TRUE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE(user_id, skill_id)
);

-- Skills required for specific shifts
CREATE TABLE shift_required_skills (
    id SERIAL PRIMARY KEY,
    shift_id INTEGER NOT NULL REFERENCES shifts(id) ON DELETE CASCADE,
    skill_id INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    is_required BOOLEAN NOT NULL DEFAULT TRUE,
    -- TRUE = required, FALSE = preferred
    minimum_proficiency VARCHAR(50) NOT NULL DEFAULT 'Beginner',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(shift_id, skill_id)
);

-- User schedule preferences (availability)
CREATE TABLE user_shift_schedules (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    -- Weekly availability (time in HH:MM format)
    monday_start TIME,
    monday_end TIME,
    tuesday_start TIME,
    tuesday_end TIME,
    wednesday_start TIME,
    wednesday_end TIME,
    thursday_start TIME,
    thursday_end TIME,
    friday_start TIME,
    friday_end TIME,
    saturday_start TIME,
    saturday_end TIME,
    sunday_start TIME,
    sunday_end TIME,
    -- Hour constraints
    max_hours_per_week INTEGER,
    min_hours_per_week INTEGER,
    is_available_for_overtime BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, company_id)
);

-- Shift assignment proposals (for manager-to-user assignment workflow)
CREATE TABLE shift_assignments (
    id SERIAL PRIMARY KEY,
    shift_id INTEGER NOT NULL REFERENCES shifts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assignment_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- 'pending', 'accepted', 'declined', 'cancelled', 'expired'
    acceptance_deadline TIMESTAMPTZ,
    response VARCHAR(50),
    -- 'accept', 'decline'
    response_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(shift_id, user_id) -- Only one assignment per shift per user
);

-- Shift swap responses (for when someone responds to a swap request)
CREATE TABLE shift_swap_responses (
    id SERIAL PRIMARY KEY,
    swap_id INTEGER NOT NULL REFERENCES shift_swaps(id) ON DELETE CASCADE,
    responding_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    response_type VARCHAR(50) NOT NULL,
    -- 'interested', 'accepted', 'declined'
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(swap_id, responding_user_id)
);

-- Indexes for performance
CREATE INDEX idx_skills_category ON skills(category);

CREATE INDEX idx_skills_is_certification ON skills(is_certification);

CREATE INDEX idx_user_skills_user_id ON user_skills(user_id);

CREATE INDEX idx_user_skills_skill_id ON user_skills(skill_id);

CREATE INDEX idx_user_skills_proficiency_level ON user_skills(proficiency_level);

CREATE INDEX idx_user_skills_is_active ON user_skills(is_active);

CREATE INDEX idx_shift_required_skills_shift_id ON shift_required_skills(shift_id);

CREATE INDEX idx_shift_required_skills_skill_id ON shift_required_skills(skill_id);

CREATE INDEX idx_user_shift_schedules_user_id ON user_shift_schedules(user_id);

CREATE INDEX idx_user_shift_schedules_company_id ON user_shift_schedules(company_id);

CREATE INDEX idx_shift_assignments_shift_id ON shift_assignments(shift_id);

CREATE INDEX idx_shift_assignments_user_id ON shift_assignments(user_id);

CREATE INDEX idx_shift_assignments_assigned_by ON shift_assignments(assigned_by);

CREATE INDEX idx_shift_assignments_status ON shift_assignments(assignment_status);

CREATE INDEX idx_shift_assignments_deadline ON shift_assignments(acceptance_deadline);

CREATE INDEX idx_shift_swap_responses_swap_id ON shift_swap_responses(swap_id);

CREATE INDEX idx_shift_swap_responses_user_id ON shift_swap_responses(responding_user_id);

CREATE INDEX idx_shift_swap_responses_response_type ON shift_swap_responses(response_type);
