-- Create invite tokens table
CREATE TABLE IF NOT EXISTS invite_tokens (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    token TEXT UNIQUE NOT NULL,
    inviter_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'employee',
    team_id INTEGER,
    expires_at DATETIME NOT NULL,
    used_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (inviter_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE
    SET
        NULL
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_invite_tokens_token ON invite_tokens(token);

CREATE INDEX IF NOT EXISTS idx_invite_tokens_email ON invite_tokens(email);

CREATE INDEX IF NOT EXISTS idx_invite_tokens_inviter_id ON invite_tokens(inviter_id);

CREATE INDEX IF NOT EXISTS idx_invite_tokens_expires_at ON invite_tokens(expires_at);

CREATE INDEX IF NOT EXISTS idx_invite_tokens_used_at ON invite_tokens(used_at);
