-- Scheduling system: shifts, claims, assignments, and user schedules
-- This migration creates the core scheduling functionality
-- Shifts (scheduled work periods)
CREATE TABLE
    shifts (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        title VARCHAR(255) NOT NULL,
        description TEXT,
        location_id UUID NOT NULL REFERENCES locations (id) ON DELETE CASCADE,
        team_id UUID REFERENCES teams (id) ON DELETE SET NULL,
        start_time TIMESTAMPTZ NOT NULL,
        end_time TIMESTAMPTZ NOT NULL,
        min_duration_minutes INTEGER,
        max_duration_minutes INTEGER,
        max_people INTEGER,
        status VARCHAR(50) NOT NULL DEFAULT 'open',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Shift claims
CREATE TABLE
    shift_claims (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        shift_id UUID NOT NULL REFERENCES shifts (id) ON DELETE CASCADE,
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        status VARCHAR(50) NOT NULL DEFAULT 'pending',
        actioned_by UUID REFERENCES users (id) ON DELETE SET NULL,
        action_notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (shift_id, user_id)
    );

-- Shift assignments
CREATE TABLE
    shift_assignments (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        shift_id UUID NOT NULL REFERENCES shifts (id) ON DELETE CASCADE,
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        assigned_by UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        assignment_status VARCHAR(50) NOT NULL DEFAULT 'pending',
        acceptance_deadline TIMESTAMPTZ,
        response VARCHAR(50),
        response_notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (shift_id, user_id)
    );

-- User shift schedules (availability preferences)
CREATE TABLE
    user_shift_schedules (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
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
        max_hours_per_week INTEGER,
        min_hours_per_week INTEGER,
        is_available_for_overtime BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (user_id, company_id)
    );

-- Indexes for performance
CREATE INDEX idx_shifts_company_id ON shifts (company_id);

CREATE INDEX idx_shifts_location_id ON shifts (location_id);

CREATE INDEX idx_shifts_team_id ON shifts (team_id);

CREATE INDEX idx_shifts_start_time ON shifts (start_time);

CREATE INDEX idx_shifts_end_time ON shifts (end_time);

CREATE INDEX idx_shifts_status ON shifts (status);

CREATE INDEX idx_shift_claims_shift_id ON shift_claims (shift_id);

CREATE INDEX idx_shift_claims_user_id ON shift_claims (user_id);

CREATE INDEX idx_shift_claims_status ON shift_claims (status);

CREATE INDEX idx_shift_assignments_shift_id ON shift_assignments (shift_id);

CREATE INDEX idx_shift_assignments_user_id ON shift_assignments (user_id);

CREATE INDEX idx_shift_assignments_assigned_by ON shift_assignments (assigned_by);

CREATE INDEX idx_shift_assignments_status ON shift_assignments (assignment_status);

CREATE INDEX idx_user_shift_schedules_user_id ON user_shift_schedules (user_id);

CREATE INDEX idx_user_shift_schedules_company_id ON user_shift_schedules (company_id);

-- Add foreign key constraint for shift_required_skills (created in migration 3)
ALTER TABLE shift_required_skills ADD CONSTRAINT shift_required_skills_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts (id) ON DELETE CASCADE;
