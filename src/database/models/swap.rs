use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwapResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub swap_type: String,
    pub requested_by: SwapUser,
    pub original_shift: SwapShift,
    pub status: String,
    pub reason: String,
    pub request_date: NaiveDateTime,
    pub responses: Option<Vec<SwapResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapUser {
    pub id: String,
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapShift {
    pub id: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub department: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub id: String,
    pub user: SwapUser,
    pub status: String,
}

impl From<ShiftSwap> for ShiftSwapResponse {
    fn from(swap: ShiftSwap) -> Self {
        Self {
            id: swap.id.to_string(),
            swap_type: swap.swap_type.to_string(),
            requested_by: SwapUser {
                id: swap.requesting_user_id.clone(),
                name: "Unknown User".to_string(), // Will be filled by join query
                avatar: "".to_string(),           // Will be filled by join query
            },
            original_shift: SwapShift {
                id: swap.original_shift_id.to_string(),
                start_time: NaiveDateTime::default(), // Will be filled by join query
                end_time: NaiveDateTime::default(),   // Will be filled by join query
                department: "Unknown".to_string(),    // Will be filled by join query
            },
            status: swap.status.to_string(),
            reason: swap.notes.unwrap_or_default(),
            request_date: swap.created_at,
            responses: None, // Will be filled by separate query if needed
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwapResponseRecord {
    pub id: i64,
    pub swap_id: i64,
    pub responding_user_id: String,
    pub status: ShiftSwapResponseStatus,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwapResponseInput {
    pub swap_id: i64,
    pub responding_user_id: String,
    pub status: ShiftSwapResponseStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShiftSwapResponseStatus {
    Pending,
    Interested,
    NotInterested,
    Accepted,
    Declined,
}

impl std::fmt::Display for ShiftSwapResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftSwapResponseStatus::Pending => write!(f, "pending"),
            ShiftSwapResponseStatus::Interested => write!(f, "interested"),
            ShiftSwapResponseStatus::NotInterested => write!(f, "not_interested"),
            ShiftSwapResponseStatus::Accepted => write!(f, "accepted"),
            ShiftSwapResponseStatus::Declined => write!(f, "declined"),
        }
    }
}

impl std::str::FromStr for ShiftSwapResponseStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ShiftSwapResponseStatus::Pending),
            "interested" => Ok(ShiftSwapResponseStatus::Interested),
            "not_interested" => Ok(ShiftSwapResponseStatus::NotInterested),
            "accepted" => Ok(ShiftSwapResponseStatus::Accepted),
            "declined" => Ok(ShiftSwapResponseStatus::Declined),
            _ => Err(format!("Invalid shift swap response status: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for ShiftSwapResponseStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ShiftSwapResponseStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ShiftSwapResponseStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<ShiftSwapResponseStatus>().map_err(|e| e.into())
    }
}
