use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location_id: Uuid,         // UUID type
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUpdateTeamInput {
    pub name: String,
    pub description: Option<String>,
    pub location_id: Uuid, // UUID type
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub id: Uuid,                  // UUID type
    pub team_id: Uuid,             // UUID type
    pub user_id: Uuid,             // UUID for user references
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
}
