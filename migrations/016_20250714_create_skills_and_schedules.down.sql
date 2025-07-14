-- Rollback migration: Remove skills system and revert shift model
-- Drop new tables
DROP TABLE IF EXISTS shift_assignments;

DROP TABLE IF EXISTS user_shift_schedules;

DROP TABLE IF EXISTS shift_required_skills;

DROP TABLE IF EXISTS user_skills;

DROP TABLE IF EXISTS skills;

-- Recreate original shifts table structure
CREATE TABLE shifts_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    location_id INTEGER NOT NULL,
    team_id INTEGER,
    assigned_user_id INTEGER,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    hourly_rate REAL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (location_id) REFERENCES locations(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE
    SET
        NULL,
        FOREIGN KEY (assigned_user_id) REFERENCES users(id) ON DELETE
    SET
        NULL
);

-- Copy data back from current shifts table
INSERT INTO
    shifts_old (
        id,
        title,
        description,
        location_id,
        team_id,
        start_time,
        end_time,
        status,
        created_at,
        updated_at
    )
SELECT
    id,
    title,
    description,
    location_id,
    team_id,
    start_time,
    end_time,
    status,
    created_at,
    updated_at
FROM
    shifts;

-- Drop current table and rename old one
DROP TABLE shifts;

ALTER TABLE
    shifts_old RENAME TO shifts;

-- Recreate original indexes
CREATE INDEX IF NOT EXISTS idx_shifts_location_id ON shifts(location_id);

CREATE INDEX IF NOT EXISTS idx_shifts_team_id ON shifts(team_id);

CREATE INDEX IF NOT EXISTS idx_shifts_assigned_user_id ON shifts(assigned_user_id);

CREATE INDEX IF NOT EXISTS idx_shifts_start_time ON shifts(start_time);

CREATE INDEX IF NOT EXISTS idx_shifts_status ON shifts(status);
