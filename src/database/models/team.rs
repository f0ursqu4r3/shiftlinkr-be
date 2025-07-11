use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInput {
    pub name: String,
    pub description: Option<String>,
    pub location_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamMember {
    pub id: i64,
    pub team_id: i64,
    pub user_id: i64,
    pub created_at: NaiveDateTime,
}
