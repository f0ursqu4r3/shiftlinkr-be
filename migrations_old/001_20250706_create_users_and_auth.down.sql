-- Drop indexes
DROP INDEX IF EXISTS idx_password_reset_tokens_expires_at;

DROP INDEX IF EXISTS idx_password_reset_tokens_user_id;

DROP INDEX IF EXISTS idx_password_reset_tokens_token;

DROP INDEX IF EXISTS idx_users_email;

-- Drop tables
DROP TABLE IF EXISTS password_reset_tokens;

DROP TABLE IF EXISTS users;
