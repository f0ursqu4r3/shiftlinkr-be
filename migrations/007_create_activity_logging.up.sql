-- Activity logging system: company activity logs
-- This migration creates the audit trail system
-- Company activity logs
CREATE TABLE
    company_activity (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        user_id UUID REFERENCES users (id) ON DELETE SET NULL,
        activity_type VARCHAR(100) NOT NULL,
        entity_type VARCHAR(100) NOT NULL,
        entity_id UUID NOT NULL,
        action VARCHAR(100) NOT NULL,
        description TEXT NOT NULL,
        metadata JSONB,
        ip_address VARCHAR(45),
        user_agent TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Indexes for performance
CREATE INDEX idx_company_activity_company_id ON company_activity (company_id);

CREATE INDEX idx_company_activity_user_id ON company_activity (user_id);

CREATE INDEX idx_company_activity_activity_type ON company_activity (activity_type);

CREATE INDEX idx_company_activity_entity_type ON company_activity (entity_type);

CREATE INDEX idx_company_activity_entity_id ON company_activity (entity_id);

CREATE INDEX idx_company_activity_action ON company_activity (action);

CREATE INDEX idx_company_activity_created_at ON company_activity (created_at);
