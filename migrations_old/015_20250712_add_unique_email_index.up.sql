-- Add unique index for email on users table
-- This ensures no duplicate email addresses can be created
CREATE UNIQUE INDEX idx_users_email_unique ON users(email);
