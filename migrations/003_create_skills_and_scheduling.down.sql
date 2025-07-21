-- Drop skills and scheduling tables in reverse dependency order
DROP TABLE IF EXISTS shift_swap_responses;

DROP TABLE IF EXISTS shift_assignments;

DROP TABLE IF EXISTS user_shift_schedules;

DROP TABLE IF EXISTS shift_required_skills;

DROP TABLE IF EXISTS user_skills;

DROP TABLE IF EXISTS skills;
