-- Skills system: skills, user skills, and shift required skills
-- This migration creates the skills management system
-- Skills that can be assigned to users and required for shifts
CREATE TABLE
    skills (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        name VARCHAR(255) NOT NULL UNIQUE,
        description TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- User skills mapping with proficiency levels
CREATE TABLE
    user_skills (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        skill_id UUID NOT NULL REFERENCES skills (id) ON DELETE CASCADE,
        proficiency_level VARCHAR(50) NOT NULL DEFAULT 'Beginner',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (user_id, skill_id)
    );

-- Skills required for specific shifts
CREATE TABLE
    shift_required_skills (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        shift_id UUID NOT NULL, -- REFERENCES shifts(id) ON DELETE CASCADE - will be added in migration 4
        skill_id UUID NOT NULL REFERENCES skills (id) ON DELETE CASCADE,
        required_level VARCHAR(50) NOT NULL DEFAULT 'Beginner',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (shift_id, skill_id)
    );

-- Indexes for performance
CREATE INDEX idx_skills_company_id ON skills (company_id);

CREATE INDEX idx_skills_name ON skills (name);

CREATE INDEX idx_user_skills_user_id ON user_skills (user_id);

CREATE INDEX idx_user_skills_skill_id ON user_skills (skill_id);

CREATE INDEX idx_user_skills_proficiency_level ON user_skills (proficiency_level);

CREATE INDEX idx_shift_required_skills_shift_id ON shift_required_skills (shift_id);

CREATE INDEX idx_shift_required_skills_skill_id ON shift_required_skills (skill_id);
