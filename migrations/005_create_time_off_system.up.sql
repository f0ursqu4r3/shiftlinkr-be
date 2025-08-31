-- Time off system: time off requests
-- This migration creates the time off request management system
-- Time off requests
CREATE TABLE
    time_off_requests (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        start_date TIMESTAMPTZ NOT NULL,
        end_date TIMESTAMPTZ NOT NULL,
        reason TEXT,
        request_type VARCHAR(50) NOT NULL,
        status VARCHAR(50) NOT NULL DEFAULT 'pending',
        actioned_by UUID REFERENCES users (id) ON DELETE SET NULL,
        action_notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Indexes for performance
CREATE INDEX idx_time_off_requests_user_id ON time_off_requests (user_id);

CREATE INDEX idx_time_off_requests_company_id ON time_off_requests (company_id);

CREATE INDEX idx_time_off_requests_start_date ON time_off_requests (start_date);

CREATE INDEX idx_time_off_requests_end_date ON time_off_requests (end_date);

CREATE INDEX idx_time_off_requests_status ON time_off_requests (status);
