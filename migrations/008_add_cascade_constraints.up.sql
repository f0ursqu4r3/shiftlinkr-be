-- Add CASCADE constraints to prevent orphaned data when parent records are deleted
-- This migration ensures data integrity by properly handling foreign key relationships
-- 1. Activity logs - CASCADE when user deleted
ALTER TABLE activity_logs
DROP CONSTRAINT IF EXISTS activity_logs_user_id_fkey CASCADE,
ADD CONSTRAINT activity_logs_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

-- 2. Locations - CASCADE when company deleted  
ALTER TABLE locations
DROP CONSTRAINT IF EXISTS locations_company_id_fkey CASCADE,
ADD CONSTRAINT locations_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

-- 3. Teams - CASCADE when company deleted
ALTER TABLE teams
DROP CONSTRAINT IF EXISTS teams_company_id_fkey CASCADE,
ADD CONSTRAINT teams_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

-- 4. User-company associations - CASCADE on both user and company deletion
ALTER TABLE user_company
DROP CONSTRAINT IF EXISTS user_company_user_id_fkey CASCADE,
ADD CONSTRAINT user_company_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

ALTER TABLE user_company
DROP CONSTRAINT IF EXISTS user_company_company_id_fkey CASCADE,
ADD CONSTRAINT user_company_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

-- 5. Skills - CASCADE when company deleted
ALTER TABLE skills
DROP CONSTRAINT IF EXISTS skills_company_id_fkey CASCADE,
ADD CONSTRAINT skills_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

-- 6. User skills - CASCADE when user or skill deleted
ALTER TABLE user_skills
DROP CONSTRAINT IF EXISTS user_skills_user_id_fkey CASCADE,
ADD CONSTRAINT user_skills_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

ALTER TABLE user_skills
DROP CONSTRAINT IF EXISTS user_skills_skill_id_fkey CASCADE,
ADD CONSTRAINT user_skills_skill_id_fkey FOREIGN KEY (skill_id) REFERENCES skills (id) ON DELETE CASCADE;

-- 7. Shifts - CASCADE for company/location/team, SET NULL for optional user assignment
ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_company_id_fkey CASCADE,
ADD CONSTRAINT shifts_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_location_id_fkey CASCADE,
ADD CONSTRAINT shifts_location_id_fkey FOREIGN KEY (location_id) REFERENCES locations (id) ON DELETE CASCADE;

ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_team_id_fkey CASCADE,
ADD CONSTRAINT shifts_team_id_fkey FOREIGN KEY (team_id) REFERENCES teams (id) ON DELETE SET NULL;

ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_user_id_fkey CASCADE,
ADD CONSTRAINT shifts_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE SET NULL;

-- 8. Shift claims - CASCADE when shift or user deleted
ALTER TABLE shift_claims
DROP CONSTRAINT IF EXISTS shift_claims_shift_id_fkey CASCADE,
ADD CONSTRAINT shift_claims_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts (id) ON DELETE CASCADE;

ALTER TABLE shift_claims
DROP CONSTRAINT IF EXISTS shift_claims_user_id_fkey CASCADE,
ADD CONSTRAINT shift_claims_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

-- 9. Shift required skills - CASCADE when shift or skill deleted  
ALTER TABLE shift_required_skills
DROP CONSTRAINT IF EXISTS shift_required_skills_shift_id_fkey CASCADE,
ADD CONSTRAINT shift_required_skills_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts (id) ON DELETE CASCADE;

ALTER TABLE shift_required_skills
DROP CONSTRAINT IF EXISTS shift_required_skills_skill_id_fkey CASCADE,
ADD CONSTRAINT shift_required_skills_skill_id_fkey FOREIGN KEY (skill_id) REFERENCES skills (id) ON DELETE CASCADE;

-- 10. Shift assignments - CASCADE when shift deleted, SET NULL when user deleted
ALTER TABLE shift_assignments
DROP CONSTRAINT IF EXISTS shift_assignments_shift_id_fkey CASCADE,
ADD CONSTRAINT shift_assignments_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts (id) ON DELETE CASCADE;

ALTER TABLE shift_assignments
DROP CONSTRAINT IF EXISTS shift_assignments_user_id_fkey CASCADE,
ADD CONSTRAINT shift_assignments_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE SET NULL;

-- 11. Shift swaps - CASCADE when shifts deleted, SET NULL when users deleted
ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_requesting_user_id_fkey CASCADE,
ADD CONSTRAINT shift_swaps_requesting_user_id_fkey FOREIGN KEY (requesting_user_id) REFERENCES users (id) ON DELETE SET NULL;

ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_original_shift_id_fkey CASCADE,
ADD CONSTRAINT shift_swaps_original_shift_id_fkey FOREIGN KEY (original_shift_id) REFERENCES shifts (id) ON DELETE CASCADE;

ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_target_shift_id_fkey CASCADE,
ADD CONSTRAINT shift_swaps_target_shift_id_fkey FOREIGN KEY (target_shift_id) REFERENCES shifts (id) ON DELETE CASCADE;

ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_target_user_id_fkey CASCADE,
ADD CONSTRAINT shift_swaps_target_user_id_fkey FOREIGN KEY (target_user_id) REFERENCES users (id) ON DELETE SET NULL;

-- 12. Time off requests - CASCADE when user deleted
ALTER TABLE time_off_requests
DROP CONSTRAINT IF EXISTS time_off_requests_user_id_fkey CASCADE,
ADD CONSTRAINT time_off_requests_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

-- 13. User schedules - CASCADE when user or company deleted
ALTER TABLE user_schedules
DROP CONSTRAINT IF EXISTS user_schedules_user_id_fkey CASCADE,
ADD CONSTRAINT user_schedules_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

ALTER TABLE user_schedules
DROP CONSTRAINT IF EXISTS user_schedules_company_id_fkey CASCADE,
ADD CONSTRAINT user_schedules_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

-- 14. Wage history - CASCADE when user or company deleted
ALTER TABLE wage_history
DROP CONSTRAINT IF EXISTS wage_history_user_id_fkey CASCADE,
ADD CONSTRAINT wage_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

ALTER TABLE wage_history
DROP CONSTRAINT IF EXISTS wage_history_company_id_fkey CASCADE,
ADD CONSTRAINT wage_history_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

-- 15. Handle password_reset_tokens if they exist (from auth flow)
DO $$
    BEGIN IF EXISTS (
        SELECT
            1
        FROM
            information_schema.tables
        WHERE
            table_name = 'password_reset_tokens'
    ) THEN
        ALTER TABLE password_reset_tokens
        DROP CONSTRAINT IF EXISTS password_reset_tokens_user_id_fkey CASCADE,
        ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;
    END IF;
END $$;

-- 16. Handle invites table if it exists
DO $$
    BEGIN IF EXISTS (
        SELECT
            1
        FROM
            information_schema.tables
        WHERE
            table_name = 'invites'
    ) THEN
        ALTER TABLE invites
        DROP CONSTRAINT IF EXISTS invites_company_id_fkey CASCADE,
        ADD CONSTRAINT invites_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id) ON DELETE CASCADE;

        ALTER TABLE invites
        DROP CONSTRAINT IF EXISTS invites_invited_by_fkey CASCADE,
        ADD CONSTRAINT invites_invited_by_fkey FOREIGN KEY (invited_by) REFERENCES users (id) ON DELETE SET NULL;
    END IF;
END $$;
