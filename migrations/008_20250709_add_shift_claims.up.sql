-- Add shift_claims table for tracking shift claim requests and approvals
CREATE TABLE IF NOT EXISTS shift_claims (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shift_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    approved_by TEXT,
    approval_notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (shift_id) REFERENCES shifts(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (approved_by) REFERENCES users(id) ON DELETE
    SET
        NULL
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_shift_claims_shift_id ON shift_claims(shift_id);

CREATE INDEX IF NOT EXISTS idx_shift_claims_user_id ON shift_claims(user_id);

CREATE INDEX IF NOT EXISTS idx_shift_claims_status ON shift_claims(status);

CREATE INDEX IF NOT EXISTS idx_shift_claims_created_at ON shift_claims(created_at);

CREATE INDEX IF NOT EXISTS idx_shift_claims_approved_by ON shift_claims(approved_by);
