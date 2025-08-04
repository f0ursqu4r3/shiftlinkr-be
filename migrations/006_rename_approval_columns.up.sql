-- Rename approval columns to action columns for consistency across tables
-- This provides better naming for actions that can be both approvals and denials
-- Rename columns in time_off_requests table
ALTER TABLE time_off_requests
RENAME COLUMN approved_by TO actioned_by;

ALTER TABLE time_off_requests
RENAME COLUMN approval_notes TO action_notes;

-- Rename columns in shift_swaps table  
ALTER TABLE shift_swaps
RENAME COLUMN approved_by TO actioned_by;

ALTER TABLE shift_swaps
RENAME COLUMN approval_notes TO action_notes;

-- Rename columns in shift_claims table
ALTER TABLE shift_claims
RENAME COLUMN approved_by TO actioned_by;

ALTER TABLE shift_claims
RENAME COLUMN approval_notes TO action_notes;
