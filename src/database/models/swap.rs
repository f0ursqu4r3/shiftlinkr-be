use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShiftSwap {
    pub id: i64,
    pub requesting_user_id: String,
    pub original_shift_id: i64,
    pub target_user_id: Option<String>,
    pub target_shift_id: Option<i64>,
    pub notes: Option<String>,
    pub swap_type: ShiftSwapType,
    pub status: ShiftSwapStatus,
    pub approved_by: Option<String>,
    pub approval_notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftSwapInput {
    pub original_shift_id: i64,
    pub requesting_user_id: String,
    pub target_user_id: Option<String>,
    pub target_shift_id: Option<i64>,
    pub notes: Option<String>,
    pub swap_type: ShiftSwapType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShiftSwapType {
    Open,     // Open to any qualified employee
    Targeted, // Targeted to specific employee
}

impl std::fmt::Display for ShiftSwapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftSwapType::Open => write!(f, "open"),
            ShiftSwapType::Targeted => write!(f, "targeted"),
        }
    }
}

impl std::str::FromStr for ShiftSwapType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(ShiftSwapType::Open),
            "targeted" => Ok(ShiftSwapType::Targeted),
            _ => Err(format!("Invalid shift swap type: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for ShiftSwapType {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ShiftSwapType {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ShiftSwapType {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<ShiftSwapType>().map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShiftSwapStatus {
    Open,
    Pending,
    Approved,
    Denied,
    Completed,
    Cancelled,
}

impl std::fmt::Display for ShiftSwapStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftSwapStatus::Open => write!(f, "open"),
            ShiftSwapStatus::Pending => write!(f, "pending"),
            ShiftSwapStatus::Approved => write!(f, "approved"),
            ShiftSwapStatus::Denied => write!(f, "denied"),
            ShiftSwapStatus::Completed => write!(f, "completed"),
            ShiftSwapStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for ShiftSwapStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(ShiftSwapStatus::Open),
            "pending" => Ok(ShiftSwapStatus::Pending),
            "approved" => Ok(ShiftSwapStatus::Approved),
            "denied" => Ok(ShiftSwapStatus::Denied),
            "completed" => Ok(ShiftSwapStatus::Completed),
            "cancelled" => Ok(ShiftSwapStatus::Cancelled),
            _ => Err(format!("Invalid shift swap status: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for ShiftSwapStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ShiftSwapStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ShiftSwapStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<ShiftSwapStatus>().map_err(|e| e.into())
    }
}
