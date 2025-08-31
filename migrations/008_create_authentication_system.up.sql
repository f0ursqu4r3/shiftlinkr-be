-- Authentication system: password reset tokens
-- This migration creates the password reset functionality
-- Password reset tokens
CREATE TABLE
    password_reset_tokens (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        token VARCHAR(255) NOT NULL UNIQUE,
        expires_at TIMESTAMPTZ NOT NULL,
        used_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Indexes for performance
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens (user_id);

CREATE INDEX idx_password_reset_tokens_token ON password_reset_tokens (token);

CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens (expires_at);
