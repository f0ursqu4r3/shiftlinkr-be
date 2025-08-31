-- Rollback scheduling system
-- Remove foreign key constraint for shift_required_skills
ALTER TABLE shift_required_skills
DROP CONSTRAINT IF EXISTS shift_required_skills_shift_id_fkey;

DROP TABLE IF EXISTS user_shift_schedules;

DROP TABLE IF EXISTS shift_assignments;

DROP TABLE IF EXISTS shift_claims;

DROP TABLE IF EXISTS shifts;
