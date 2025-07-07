-- Drop indexes first
DROP INDEX IF EXISTS idx_shifts_status;

DROP INDEX IF EXISTS idx_shifts_start_time;

DROP INDEX IF EXISTS idx_shifts_assigned_user_id;

DROP INDEX IF EXISTS idx_shifts_team_id;

DROP INDEX IF EXISTS idx_shifts_location_id;

DROP INDEX IF EXISTS idx_team_members_user_id;

DROP INDEX IF EXISTS idx_team_members_team_id;

DROP INDEX IF EXISTS idx_teams_location_id;

-- Drop tables in reverse order (due to foreign key constraints)
DROP TABLE IF EXISTS shifts;

DROP TABLE IF EXISTS team_members;

DROP TABLE IF EXISTS teams;

DROP TABLE IF EXISTS locations;
