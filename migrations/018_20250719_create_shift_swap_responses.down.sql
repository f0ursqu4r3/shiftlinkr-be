-- Drop shift_swap_responses table
DROP INDEX IF EXISTS idx_shift_swap_responses_created_at;

DROP INDEX IF EXISTS idx_shift_swap_responses_status;

DROP INDEX IF EXISTS idx_shift_swap_responses_user_id;

DROP INDEX IF EXISTS idx_shift_swap_responses_swap_id;

DROP TABLE IF EXISTS shift_swap_responses;
