-- Create locations table
CREATE TABLE IF NOT EXISTS locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    address TEXT,
    phone TEXT,
    email TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create teams table
CREATE TABLE IF NOT EXISTS teams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    location_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (location_id) REFERENCES locations(id) ON DELETE CASCADE
);

-- Create team_members table
CREATE TABLE IF NOT EXISTS team_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(team_id, user_id)
);

-- Create shifts table
CREATE TABLE IF NOT EXISTS shifts (
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

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_teams_location_id ON teams(location_id);

CREATE INDEX IF NOT EXISTS idx_team_members_team_id ON team_members(team_id);

CREATE INDEX IF NOT EXISTS idx_team_members_user_id ON team_members(user_id);

CREATE INDEX IF NOT EXISTS idx_shifts_location_id ON shifts(location_id);

CREATE INDEX IF NOT EXISTS idx_shifts_team_id ON shifts(team_id);

CREATE INDEX IF NOT EXISTS idx_shifts_assigned_user_id ON shifts(assigned_user_id);

CREATE INDEX IF NOT EXISTS idx_shifts_start_time ON shifts(start_time);

CREATE INDEX IF NOT EXISTS idx_shifts_status ON shifts(status);
