-- Remove indexes
DROP INDEX IF EXISTS idx_invite_tokens_status ON invite_tokens;

-- Remove invite status
ALTER TABLE invite_tokens
DROP COLUMN status;
