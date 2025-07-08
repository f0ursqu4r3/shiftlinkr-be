-- Create time_off_requests table
CREATE TABLE time_off_requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    start_date DATETIME NOT NULL,
    end_date DATETIME NOT NULL,
    reason TEXT,
    request_type TEXT NOT NULL DEFAULT 'vacation',
    status TEXT NOT NULL DEFAULT 'pending',
    approved_by TEXT,
    approval_notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (approved_by) REFERENCES users(id) ON DELETE SET NULL
);

-- Create indexes for better performance
CREATE INDEX idx_time_off_requests_user_id ON time_off_requests(user_id);
CREATE INDEX idx_time_off_requests_status ON time_off_requests(status);
CREATE INDEX idx_time_off_requests_start_date ON time_off_requests(start_date);
CREATE INDEX idx_time_off_requests_end_date ON time_off_requests(end_date);
CREATE INDEX idx_time_off_requests_approved_by ON time_off_requests(approved_by);

-- Create shift_swaps table
CREATE TABLE shift_swaps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    requesting_user_id TEXT NOT NULL,
    original_shift_id INTEGER NOT NULL,
    target_user_id TEXT,
    target_shift_id INTEGER,
    notes TEXT,
    swap_type TEXT NOT NULL DEFAULT 'open',
    status TEXT NOT NULL DEFAULT 'pending',
    approved_by TEXT,
    approval_notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (requesting_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (original_shift_id) REFERENCES shifts(id) ON DELETE CASCADE,
    FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (target_shift_id) REFERENCES shifts(id) ON DELETE SET NULL,
    FOREIGN KEY (approved_by) REFERENCES users(id) ON DELETE SET NULL
);

-- Create indexes for better performance
CREATE INDEX idx_shift_swaps_requesting_user_id ON shift_swaps(requesting_user_id);
CREATE INDEX idx_shift_swaps_original_shift_id ON shift_swaps(original_shift_id);
CREATE INDEX idx_shift_swaps_target_user_id ON shift_swaps(target_user_id);
CREATE INDEX idx_shift_swaps_target_shift_id ON shift_swaps(target_shift_id);
CREATE INDEX idx_shift_swaps_status ON shift_swaps(status);
CREATE INDEX idx_shift_swaps_approved_by ON shift_swaps(approved_by);
