-- Organizational structure: locations, teams, and team memberships
-- This migration creates the hierarchical structure for companies
-- Locations within companies
CREATE TABLE
    locations (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        name VARCHAR(255) NOT NULL,
        address TEXT,
        phone VARCHAR(50),
        email VARCHAR(255),
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Teams within locations
CREATE TABLE
    teams (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        name VARCHAR(255) NOT NULL,
        description TEXT,
        location_id UUID NOT NULL REFERENCES locations (id) ON DELETE CASCADE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Team memberships
CREATE TABLE
    team_members (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        team_id UUID NOT NULL REFERENCES teams (id) ON DELETE CASCADE,
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (team_id, user_id)
    );

-- Indexes for performance
CREATE INDEX idx_locations_company_id ON locations (company_id);

CREATE INDEX idx_locations_name ON locations (name);

CREATE INDEX idx_teams_location_id ON teams (location_id);

CREATE INDEX idx_teams_name ON teams (name);

CREATE INDEX idx_team_members_team_id ON team_members (team_id);

CREATE INDEX idx_team_members_user_id ON team_members (user_id);
