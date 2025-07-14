use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Shift {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub team_id: Option<i64>,
    pub assigned_user_id: Option<i64>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub hourly_rate: Option<f64>,
    pub status: ShiftStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftInput {
    pub title: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub team_id: Option<i64>,
    pub assigned_user_id: Option<i64>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub hourly_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShiftStatus {
    Open,
    Assigned,
    Completed,
    Cancelled,
}

impl std::fmt::Display for ShiftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftStatus::Open => write!(f, "open"),
            ShiftStatus::Assigned => write!(f, "assigned"),
            ShiftStatus::Completed => write!(f, "completed"),
            ShiftStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for ShiftStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(ShiftStatus::Open),
            "assigned" => Ok(ShiftStatus::Assigned),
            "completed" => Ok(ShiftStatus::Completed),
            "cancelled" => Ok(ShiftStatus::Cancelled),
            _ => Err(format!("Invalid shift status: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for ShiftStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ShiftStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ShiftStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<ShiftStatus>().map_err(|e| e.into())
    }
}

impl Default for ShiftStatus {
    fn default() -> Self {
        ShiftStatus::Open
    }
}

// Shift Claim models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShiftClaim {
    pub id: i64,
    pub shift_id: i64,
    pub user_id: String,
    pub status: ShiftClaimStatus,
    pub approved_by: Option<String>,
    pub approval_notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftClaimInput {
    pub shift_id: i64,
    pub user_id: String,
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

impl sqlx::Type<sqlx::Sqlite> for ShiftClaimStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ShiftClaimStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ShiftClaimStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<ShiftClaimStatus>().map_err(|e| e.into())
    }
}

impl Default for ShiftClaimStatus {
    fn default() -> Self {
        ShiftClaimStatus::Pending
    }
}
