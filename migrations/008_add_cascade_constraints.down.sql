-- Rollback CASCADE constraints to original foreign key constraints without CASCADE
-- This reverts the migration by removing CASCADE behavior
-- 1. Activity logs - Remove CASCADE
ALTER TABLE activity_logs
DROP CONSTRAINT IF EXISTS activity_logs_user_id_fkey,
ADD CONSTRAINT activity_logs_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

-- 2. Locations - Remove CASCADE
ALTER TABLE locations
DROP CONSTRAINT IF EXISTS locations_company_id_fkey,
ADD CONSTRAINT locations_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

-- 3. Teams - Remove CASCADE
ALTER TABLE teams
DROP CONSTRAINT IF EXISTS teams_company_id_fkey,
ADD CONSTRAINT teams_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

-- 4. User-company associations - Remove CASCADE
ALTER TABLE user_company
DROP CONSTRAINT IF EXISTS user_company_user_id_fkey,
ADD CONSTRAINT user_company_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

ALTER TABLE user_company
DROP CONSTRAINT IF EXISTS user_company_company_id_fkey,
ADD CONSTRAINT user_company_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

-- 5. Skills - Remove CASCADE
ALTER TABLE skills
DROP CONSTRAINT IF EXISTS skills_company_id_fkey,
ADD CONSTRAINT skills_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

-- 6. User skills - Remove CASCADE
ALTER TABLE user_skills
DROP CONSTRAINT IF EXISTS user_skills_user_id_fkey,
ADD CONSTRAINT user_skills_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

ALTER TABLE user_skills
DROP CONSTRAINT IF EXISTS user_skills_skill_id_fkey,
ADD CONSTRAINT user_skills_skill_id_fkey FOREIGN KEY (skill_id) REFERENCES skills (id);

-- 7. Shifts - Remove CASCADE/SET NULL behavior
ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_company_id_fkey,
ADD CONSTRAINT shifts_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_location_id_fkey,
ADD CONSTRAINT shifts_location_id_fkey FOREIGN KEY (location_id) REFERENCES locations (id);

ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_team_id_fkey,
ADD CONSTRAINT shifts_team_id_fkey FOREIGN KEY (team_id) REFERENCES teams (id);

ALTER TABLE shifts
DROP CONSTRAINT IF EXISTS shifts_user_id_fkey,
ADD CONSTRAINT shifts_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

-- 8. Shift claims - Remove CASCADE
ALTER TABLE shift_claims
DROP CONSTRAINT IF EXISTS shift_claims_shift_id_fkey,
ADD CONSTRAINT shift_claims_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts (id);

ALTER TABLE shift_claims
DROP CONSTRAINT IF EXISTS shift_claims_user_id_fkey,
ADD CONSTRAINT shift_claims_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

-- 9. Shift required skills - Remove CASCADE
ALTER TABLE shift_required_skills
DROP CONSTRAINT IF EXISTS shift_required_skills_shift_id_fkey,
ADD CONSTRAINT shift_required_skills_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts (id);

ALTER TABLE shift_required_skills
DROP CONSTRAINT IF EXISTS shift_required_skills_skill_id_fkey,
ADD CONSTRAINT shift_required_skills_skill_id_fkey FOREIGN KEY (skill_id) REFERENCES skills (id);

-- 10. Shift assignments - Remove CASCADE/SET NULL
ALTER TABLE shift_assignments
DROP CONSTRAINT IF EXISTS shift_assignments_shift_id_fkey,
ADD CONSTRAINT shift_assignments_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts (id);

ALTER TABLE shift_assignments
DROP CONSTRAINT IF EXISTS shift_assignments_user_id_fkey,
ADD CONSTRAINT shift_assignments_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

-- 11. Shift swaps - Remove CASCADE/SET NULL
ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_requesting_user_id_fkey,
ADD CONSTRAINT shift_swaps_requesting_user_id_fkey FOREIGN KEY (requesting_user_id) REFERENCES users (id);

ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_original_shift_id_fkey,
ADD CONSTRAINT shift_swaps_original_shift_id_fkey FOREIGN KEY (original_shift_id) REFERENCES shifts (id);

ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_target_shift_id_fkey,
ADD CONSTRAINT shift_swaps_target_shift_id_fkey FOREIGN KEY (target_shift_id) REFERENCES shifts (id);

ALTER TABLE shift_swaps
DROP CONSTRAINT IF EXISTS shift_swaps_target_user_id_fkey,
ADD CONSTRAINT shift_swaps_target_user_id_fkey FOREIGN KEY (target_user_id) REFERENCES users (id);

-- 12. Time off requests - Remove CASCADE
ALTER TABLE time_off_requests
DROP CONSTRAINT IF EXISTS time_off_requests_user_id_fkey,
ADD CONSTRAINT time_off_requests_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

-- 13. User schedules - Remove CASCADE
ALTER TABLE user_schedules
DROP CONSTRAINT IF EXISTS user_schedules_user_id_fkey,
ADD CONSTRAINT user_schedules_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

ALTER TABLE user_schedules
DROP CONSTRAINT IF EXISTS user_schedules_company_id_fkey,
ADD CONSTRAINT user_schedules_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

-- 14. Wage history - Remove CASCADE
ALTER TABLE wage_history
DROP CONSTRAINT IF EXISTS wage_history_user_id_fkey,
ADD CONSTRAINT wage_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);

ALTER TABLE wage_history
DROP CONSTRAINT IF EXISTS wage_history_company_id_fkey,
ADD CONSTRAINT wage_history_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

-- 15. Handle password_reset_tokens if they exist
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
        DROP CONSTRAINT IF EXISTS password_reset_tokens_user_id_fkey,
        ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);
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
        DROP CONSTRAINT IF EXISTS invites_company_id_fkey,
        ADD CONSTRAINT invites_company_id_fkey FOREIGN KEY (company_id) REFERENCES companies (id);

        ALTER TABLE invites
        DROP CONSTRAINT IF EXISTS invites_invited_by_fkey,
        ADD CONSTRAINT invites_invited_by_fkey FOREIGN KEY (invited_by) REFERENCES users (id);
    END IF;
END $$;
