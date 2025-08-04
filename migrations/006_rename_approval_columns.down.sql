-- Revert approval column renames back to original names
-- Revert columns in time_off_requests table
ALTER TABLE time_off_requests
RENAME COLUMN actioned_by TO approved_by;

ALTER TABLE time_off_requests
RENAME COLUMN action_notes TO approval_notes;

-- Revert columns in shift_swaps table
ALTER TABLE shift_swaps
RENAME COLUMN actioned_by TO approved_by;

ALTER TABLE shift_swaps
RENAME COLUMN action_notes TO approval_notes;

-- Revert columns in shift_claims table
ALTER TABLE shift_claims
RENAME COLUMN actioned_by TO approved_by;

ALTER TABLE shift_claims
RENAME COLUMN action_notes TO approval_notes;
