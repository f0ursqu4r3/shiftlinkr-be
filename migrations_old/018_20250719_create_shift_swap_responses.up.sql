-- Create shift_swap_responses table to track responses to swap requests
CREATE TABLE IF NOT EXISTS shift_swap_responses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    swap_id INTEGER NOT NULL,
    responding_user_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (swap_id) REFERENCES shift_swaps(id) ON DELETE CASCADE,
    FOREIGN KEY (responding_user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(swap_id, responding_user_id) -- One response per user per swap
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_shift_swap_responses_swap_id ON shift_swap_responses(swap_id);

CREATE INDEX IF NOT EXISTS idx_shift_swap_responses_user_id ON shift_swap_responses(responding_user_id);

CREATE INDEX IF NOT EXISTS idx_shift_swap_responses_status ON shift_swap_responses(status);

CREATE INDEX IF NOT EXISTS idx_shift_swap_responses_created_at ON shift_swap_responses(created_at);
