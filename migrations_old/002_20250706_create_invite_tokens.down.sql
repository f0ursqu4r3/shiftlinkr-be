-- Drop indexes
DROP INDEX IF EXISTS idx_invite_tokens_used_at;

DROP INDEX IF EXISTS idx_invite_tokens_expires_at;

DROP INDEX IF EXISTS idx_invite_tokens_inviter_id;

DROP INDEX IF EXISTS idx_invite_tokens_email;

DROP INDEX IF EXISTS idx_invite_tokens_token;

-- Drop table
DROP TABLE IF EXISTS invite_tokens;
