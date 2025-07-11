use chrono::NaiveDateTime;

// Database row types that match the exact SQLite schema
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LocationRow {
    pub id: i64,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub company_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TeamRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TeamMemberRow {
    pub id: i64,
    pub team_id: i64,
    pub user_id: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ShiftRow {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub team_id: Option<i64>,
    pub assigned_user_id: Option<i64>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub hourly_rate: Option<f64>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Conversion functions
impl From<LocationRow> for super::models::Location {
    fn from(row: LocationRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            address: row.address,
            phone: row.phone,
            email: row.email,
            company_id: row.company_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<TeamRow> for super::models::Team {
    fn from(row: TeamRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            location_id: row.location_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<TeamMemberRow> for super::models::TeamMember {
    fn from(row: TeamMemberRow) -> Self {
        Self {
            id: row.id,
            team_id: row.team_id,
            user_id: row.user_id,
            created_at: row.created_at,
        }
    }
}

impl From<ShiftRow> for super::models::Shift {
    fn from(row: ShiftRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            description: row.description,
            location_id: row.location_id,
            team_id: row.team_id,
            assigned_user_id: row.assigned_user_id,
            start_time: row.start_time,
            end_time: row.end_time,
            hourly_rate: row.hourly_rate,
            status: row.status.parse().unwrap_or(super::models::ShiftStatus::Open),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
