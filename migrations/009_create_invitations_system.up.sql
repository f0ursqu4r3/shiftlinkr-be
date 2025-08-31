-- Invitations system: invite tokens
-- This migration creates the company invitation functionality
-- Invite tokens
CREATE TABLE
    invite_tokens (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        email VARCHAR(255) NOT NULL,
        token VARCHAR(255) NOT NULL UNIQUE,
        inviter_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        role VARCHAR(50) NOT NULL DEFAULT 'employee',
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        team_id UUID REFERENCES teams (id) ON DELETE SET NULL,
        expires_at TIMESTAMPTZ NOT NULL,
        used_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Indexes for performance
CREATE INDEX idx_invite_tokens_email ON invite_tokens (email);

CREATE INDEX idx_invite_tokens_token ON invite_tokens (token);

CREATE INDEX idx_invite_tokens_inviter_id ON invite_tokens (inviter_id);

CREATE INDEX idx_invite_tokens_company_id ON invite_tokens (company_id);

CREATE INDEX idx_invite_tokens_team_id ON invite_tokens (team_id);

CREATE INDEX idx_invite_tokens_expires_at ON invite_tokens (expires_at);
