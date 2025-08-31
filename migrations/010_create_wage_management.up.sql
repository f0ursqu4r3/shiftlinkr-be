-- Wage management system: wage history
-- This migration creates the wage tracking functionality
-- Wage history
CREATE TABLE
    wage_history (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        hourly_rate DECIMAL(10, 2) NOT NULL,
        overtime_rate_multiplier DECIMAL(3, 2),
        effective_date DATE NOT NULL,
        end_date DATE,
        changed_by UUID REFERENCES users (id) ON DELETE SET NULL,
        change_reason TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Indexes for performance
CREATE INDEX idx_wage_history_user_id ON wage_history (user_id);

CREATE INDEX idx_wage_history_company_id ON wage_history (company_id);

CREATE INDEX idx_wage_history_effective_date ON wage_history (effective_date);

CREATE INDEX idx_wage_history_end_date ON wage_history (end_date);
