-- Remove shift_claims table and indexes
DROP INDEX IF EXISTS idx_shift_claims_approved_by;

DROP INDEX IF EXISTS idx_shift_claims_created_at;

DROP INDEX IF EXISTS idx_shift_claims_status;

DROP INDEX IF EXISTS idx_shift_claims_user_id;

DROP INDEX IF EXISTS idx_shift_claims_shift_id;

DROP TABLE IF EXISTS shift_claims;
