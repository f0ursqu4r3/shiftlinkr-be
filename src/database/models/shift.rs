use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Shift {
    pub id: Uuid,         // UUID primary key
    pub company_id: Uuid, // UUID for company references
    pub title: String,
    pub description: Option<String>,
    pub location_id: Uuid,                 // UUID for location references
    pub team_id: Option<Uuid>,             // UUID for team references
    pub start_time: DateTime<Utc>,         // TIMESTAMPTZ for datetime with timezone
    pub end_time: DateTime<Utc>,           // TIMESTAMPTZ for datetime with timezone
    pub min_duration_minutes: Option<i32>, // INTEGER maps to i32
    pub max_duration_minutes: Option<i32>, // INTEGER maps to i32
    pub max_people: Option<i32>,           // INTEGER maps to i32
    pub status: ShiftStatus,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftInput {
    pub company_id: Uuid, // UUID for company references
    pub title: String,
    pub description: Option<String>,
    pub location_id: Uuid,                 // UUID for location references
    pub team_id: Option<Uuid>,             // UUID for team references
    pub start_time: DateTime<Utc>,         // TIMESTAMPTZ
    pub end_time: DateTime<Utc>,           // TIMESTAMPTZ
    pub min_duration_minutes: Option<i32>, // INTEGER maps to i32
    pub max_duration_minutes: Option<i32>, // INTEGER maps to i32
    pub max_people: Option<i32>,           // INTEGER maps to i32
    pub status: ShiftStatus,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ShiftStatus {
        Open => "open",
        Assigned => "assigned",
        Completed => "completed",
        Cancelled => "cancelled",
    }
}

impl Default for ShiftStatus {
    fn default() -> Self {
        ShiftStatus::Open
    }
}

// Shift Claim models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftClaim {
    pub id: Uuid,       // UUID primary key
    pub shift_id: Uuid, // UUID for shift references
    pub user_id: Uuid,  // UUID for user references
    pub status: ShiftClaimStatus,
    pub approved_by: Option<Uuid>, // UUID for user references
    pub approval_notes: Option<String>,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftClaimInput {
    pub shift_id: Uuid, // UUID for shift references
    pub user_id: Uuid,  // UUID for user references
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ShiftClaimStatus {
        Pending => "pending",
        Approved => "approved",
        Rejected => "rejected",
        Cancelled => "cancelled",
    }
}

impl Default for ShiftClaimStatus {
    fn default() -> Self {
        ShiftClaimStatus::Pending
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftQuery {
    #[serde(flatten)]
    pub query_type: ShiftQueryType,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "id")]
#[serde(rename_all = "camelCase")]
pub enum ShiftQueryType {
    Location(Uuid),
    Team(Uuid),
    User(Uuid),
    Company(Uuid),
}
