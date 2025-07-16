-- Create shift proposal assignments table for the assignment/acceptance workflow
-- This is separate from the shift_assignments table which is for scheduled assignments
CREATE TABLE IF NOT EXISTS shift_proposal_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shift_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    assigned_by TEXT NOT NULL,
    assignment_status TEXT NOT NULL DEFAULT 'pending', -- "pending", "accepted", "declined", "cancelled", "expired"
    acceptance_deadline DATETIME,
    response TEXT, -- User's response when accepting/declining
    response_notes TEXT, -- Additional notes from the user
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (shift_id) REFERENCES shifts(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_by) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(shift_id, user_id) -- Only one proposal per shift per user
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_shift_proposal_assignments_shift_id ON shift_proposal_assignments(shift_id);

CREATE INDEX IF NOT EXISTS idx_shift_proposal_assignments_user_id ON shift_proposal_assignments(user_id);

CREATE INDEX IF NOT EXISTS idx_shift_proposal_assignments_assigned_by ON shift_proposal_assignments(assigned_by);

CREATE INDEX IF NOT EXISTS idx_shift_proposal_assignments_status ON shift_proposal_assignments(assignment_status);

CREATE INDEX IF NOT EXISTS idx_shift_proposal_assignments_deadline ON shift_proposal_assignments(acceptance_deadline);
