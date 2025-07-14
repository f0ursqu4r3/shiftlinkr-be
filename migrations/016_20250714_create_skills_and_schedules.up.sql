-- Migration: Create skills system and new shift scheduling model
-- This migration introduces:
-- 1. Skills system with user skill mappings
-- 2. New shift scheduling model (shift definitions -> user schedules -> assignments)
-- 3. Updates existing shifts table to be shift definitions
-- Create skills table
CREATE TABLE IF NOT EXISTS skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    category TEXT DEFAULT 'general',
    -- 'certification', 'equipment', 'general', 'management'
    is_certification BOOLEAN DEFAULT FALSE,
    expires BOOLEAN DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create user_skills mapping table
CREATE TABLE IF NOT EXISTS user_skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    skill_id INTEGER NOT NULL,
    proficiency_level INTEGER DEFAULT 1,
    -- 1-5 scale
    certified_date DATE,
    expiration_date DATE,
    verified_by INTEGER,
    -- user_id of manager who verified
    notes TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE,
    FOREIGN KEY (verified_by) REFERENCES users(id) ON DELETE
    SET
        NULL,
        UNIQUE(user_id, skill_id)
);

-- Create shift_required_skills mapping table
CREATE TABLE IF NOT EXISTS shift_required_skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shift_id INTEGER NOT NULL,
    skill_id INTEGER NOT NULL,
    is_required BOOLEAN DEFAULT TRUE,
    -- vs "preferred"
    minimum_proficiency INTEGER DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (shift_id) REFERENCES shifts(id) ON DELETE CASCADE,
    FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE,
    UNIQUE(shift_id, skill_id)
);

-- Create user_shift_schedules table
CREATE TABLE IF NOT EXISTS user_shift_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    shift_id INTEGER NOT NULL,
    hourly_rate REAL,
    -- User-specific pay rate for this shift
    start_time TIME NOT NULL,
    -- "09:00:00"
    end_time TIME NOT NULL,
    -- "17:00:00"
    days_of_week TEXT NOT NULL,
    -- JSON array like "[1,2,3,4,5]" for Mon-Fri
    start_date DATE NOT NULL,
    end_date DATE,
    -- NULL means indefinite
    recurrence_pattern TEXT DEFAULT 'weekly',
    -- "weekly", "bi-weekly", "monthly"
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (shift_id) REFERENCES shifts(id) ON DELETE CASCADE
);

-- Create shift_assignments table (daily assignments generated from schedules)
CREATE TABLE IF NOT EXISTS shift_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_shift_schedule_id INTEGER,
    -- Links back to the schedule that generated this
    user_id INTEGER NOT NULL,
    shift_id INTEGER NOT NULL,
    date DATE NOT NULL,
    scheduled_start_time TIME NOT NULL,
    scheduled_end_time TIME NOT NULL,
    actual_start_time TIME,
    -- Clock-in time
    actual_end_time TIME,
    -- Clock-out time
    hourly_rate REAL,
    -- Rate at time of assignment (for historical accuracy)
    status TEXT DEFAULT 'scheduled',
    -- "scheduled", "confirmed", "completed", "cancelled", "no_show"
    notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_shift_schedule_id) REFERENCES user_shift_schedules(id) ON DELETE
    SET
        NULL,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (shift_id) REFERENCES shifts(id) ON DELETE CASCADE,
        UNIQUE(user_id, date, scheduled_start_time) -- Prevent double-booking
);

-- Update existing shifts table to be shift definitions (remove user-specific fields)
-- We'll keep the existing structure but add new fields for the template model
ALTER TABLE
    shifts
ADD
    COLUMN min_duration_minutes INTEGER DEFAULT 240;

-- 4 hours default
ALTER TABLE
    shifts
ADD
    COLUMN max_duration_minutes INTEGER DEFAULT 480;

-- 8 hours default
ALTER TABLE
    shifts
ADD
    COLUMN max_people INTEGER DEFAULT 1;

-- Remove hourly_rate from shifts as it's now user-specific
-- SQLite doesn't support DROP COLUMN, so we'll create a new table and migrate data
CREATE TABLE shifts_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    location_id INTEGER NOT NULL,
    team_id INTEGER,
    start_time DATETIME NOT NULL,
    -- Keep for backward compatibility, can be used as default time
    end_time DATETIME NOT NULL,
    -- Keep for backward compatibility, can be used as default time
    min_duration_minutes INTEGER DEFAULT 240,
    max_duration_minutes INTEGER DEFAULT 480,
    max_people INTEGER DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'open',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (location_id) REFERENCES locations(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE
    SET
        NULL
);

-- Copy data from old shifts table (excluding hourly_rate and assigned_user_id)
INSERT INTO
    shifts_new (
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

-- For any existing assigned shifts, create shift_assignments records
INSERT INTO
    shift_assignments (
        user_id,
        shift_id,
        date,
        scheduled_start_time,
        scheduled_end_time,
        hourly_rate,
        status,
        created_at,
        updated_at
    )
SELECT
    assigned_user_id,
    id,
    DATE(start_time),
    TIME(start_time),
    TIME(end_time),
    hourly_rate,
    CASE
        WHEN status = 'open' THEN 'scheduled'
        WHEN status = 'assigned' THEN 'scheduled'
        ELSE status
    END,
    created_at,
    updated_at
FROM
    shifts
WHERE
    assigned_user_id IS NOT NULL;

-- Drop old table and rename new one
DROP TABLE shifts;

ALTER TABLE
    shifts_new RENAME TO shifts;

-- Recreate indexes for shifts
CREATE INDEX IF NOT EXISTS idx_shifts_location_id ON shifts(location_id);

CREATE INDEX IF NOT EXISTS idx_shifts_team_id ON shifts(team_id);

CREATE INDEX IF NOT EXISTS idx_shifts_start_time ON shifts(start_time);

CREATE INDEX IF NOT EXISTS idx_shifts_status ON shifts(status);

-- Create indexes for new tables
CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name);

CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);

CREATE INDEX IF NOT EXISTS idx_user_skills_user_id ON user_skills(user_id);

CREATE INDEX IF NOT EXISTS idx_user_skills_skill_id ON user_skills(skill_id);

CREATE INDEX IF NOT EXISTS idx_user_skills_active ON user_skills(is_active);

CREATE INDEX IF NOT EXISTS idx_shift_required_skills_shift_id ON shift_required_skills(shift_id);

CREATE INDEX IF NOT EXISTS idx_shift_required_skills_skill_id ON shift_required_skills(skill_id);

CREATE INDEX IF NOT EXISTS idx_user_shift_schedules_user_id ON user_shift_schedules(user_id);

CREATE INDEX IF NOT EXISTS idx_user_shift_schedules_shift_id ON user_shift_schedules(shift_id);

CREATE INDEX IF NOT EXISTS idx_user_shift_schedules_active ON user_shift_schedules(is_active);

CREATE INDEX IF NOT EXISTS idx_shift_assignments_user_id ON shift_assignments(user_id);

CREATE INDEX IF NOT EXISTS idx_shift_assignments_shift_id ON shift_assignments(shift_id);

CREATE INDEX IF NOT EXISTS idx_shift_assignments_date ON shift_assignments(date);

CREATE INDEX IF NOT EXISTS idx_shift_assignments_status ON shift_assignments(status);

-- Insert some default skills
INSERT INTO
    skills (
        name,
        description,
        category,
        is_certification,
        expires
    )
VALUES
    (
        'General Labor',
        'Basic work skills and reliability',
        'general',
        false,
        false
    ),
    (
        'Customer Service',
        'Customer interaction and service skills',
        'general',
        false,
        false
    ),
    (
        'Food Service',
        'Food preparation and serving experience',
        'general',
        false,
        false
    ),
    (
        'Cash Handling',
        'Register operation and money management',
        'general',
        false,
        false
    ),
    (
        'Food Safety Certified',
        'Food handler certification',
        'certification',
        true,
        true
    ),
    (
        'Opening Procedures',
        'Knowledge of store opening checklist',
        'general',
        false,
        false
    ),
    (
        'Closing Procedures',
        'Knowledge of store closing checklist',
        'general',
        false,
        false
    ),
    (
        'Team Lead',
        'Shift supervision and team management',
        'management',
        false,
        false
    ),
    (
        'POS System',
        'Point of sale system operation',
        'equipment',
        false,
        false
    ),
    (
        'Kitchen Equipment',
        'Safe operation of kitchen equipment',
        'equipment',
        false,
        false
    );
