-- Add invite status
ALTER TABLE invite_tokens
ADD COLUMN status VARCHAR(255) NOT NULL DEFAULT 'pending';

-- Indexes for performance
CREATE INDEX idx_invite_tokens_status ON invite_tokens (status);
