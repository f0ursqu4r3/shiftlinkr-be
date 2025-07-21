-- Drop organizational structure tables in reverse dependency order
ALTER TABLE
    pto_balance_history DROP CONSTRAINT IF EXISTS fk_pto_balance_history_time_off_id;

ALTER TABLE
    invite_tokens DROP CONSTRAINT IF EXISTS fk_invite_tokens_team_id;

DROP TABLE IF EXISTS shift_claims;

DROP TABLE IF EXISTS shift_swaps;

DROP TABLE IF EXISTS time_off_requests;

DROP TABLE IF EXISTS shifts;

DROP TABLE IF EXISTS team_members;

DROP TABLE IF EXISTS teams;

DROP TABLE IF EXISTS locations;
