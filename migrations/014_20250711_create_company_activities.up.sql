-- Create company activities table for audit logging
CREATE TABLE company_activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id INTEGER NOT NULL,
    user_id INTEGER,
    activity_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    description TEXT NOT NULL,
    metadata TEXT, -- JSON as TEXT in SQLite
    ip_address TEXT,
    user_agent TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX idx_company_activities_company_id ON company_activities(company_id);

CREATE INDEX idx_company_activities_created_at ON company_activities(created_at DESC);

CREATE INDEX idx_company_activities_entity ON company_activities(entity_type, entity_id);

CREATE INDEX idx_company_activities_user ON company_activities(user_id);

CREATE INDEX idx_company_activities_type ON company_activities(activity_type);
