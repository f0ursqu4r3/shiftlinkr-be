use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Shift {
    pub id: Uuid, // UUID primary key
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShiftClaimStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl std::fmt::Display for ShiftClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftClaimStatus::Pending => write!(f, "pending"),
            ShiftClaimStatus::Approved => write!(f, "approved"),
            ShiftClaimStatus::Rejected => write!(f, "rejected"),
            ShiftClaimStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for ShiftClaimStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ShiftClaimStatus::Pending),
            "approved" => Ok(ShiftClaimStatus::Approved),
            "rejected" => Ok(ShiftClaimStatus::Rejected),
            "cancelled" => Ok(ShiftClaimStatus::Cancelled),
            _ => Err(format!("Invalid shift claim status: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ShiftClaimStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ShiftClaimStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ShiftClaimStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse::<ShiftClaimStatus>().map_err(|e| e.into())
    }
}

impl Default for ShiftClaimStatus {
    fn default() -> Self {
        ShiftClaimStatus::Pending
    }
}
