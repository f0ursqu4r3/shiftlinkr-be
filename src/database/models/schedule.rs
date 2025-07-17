use chrono::{NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserShiftSchedule {
    pub id: i64,
    pub user_id: String,
    pub monday_start: Option<NaiveTime>,
    pub monday_end: Option<NaiveTime>,
    pub tuesday_start: Option<NaiveTime>,
    pub tuesday_end: Option<NaiveTime>,
    pub wednesday_start: Option<NaiveTime>,
    pub wednesday_end: Option<NaiveTime>,
    pub thursday_start: Option<NaiveTime>,
    pub thursday_end: Option<NaiveTime>,
    pub friday_start: Option<NaiveTime>,
    pub friday_end: Option<NaiveTime>,
    pub saturday_start: Option<NaiveTime>,
    pub saturday_end: Option<NaiveTime>,
    pub sunday_start: Option<NaiveTime>,
    pub sunday_end: Option<NaiveTime>,
    pub max_hours_per_week: Option<i32>,
    pub min_hours_per_week: Option<i32>,
    pub is_available_for_overtime: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserShiftScheduleInput {
    pub user_id: String,
    pub monday_start: Option<NaiveTime>,
    pub monday_end: Option<NaiveTime>,
    pub tuesday_start: Option<NaiveTime>,
    pub tuesday_end: Option<NaiveTime>,
    pub wednesday_start: Option<NaiveTime>,
    pub wednesday_end: Option<NaiveTime>,
    pub thursday_start: Option<NaiveTime>,
    pub thursday_end: Option<NaiveTime>,
    pub friday_start: Option<NaiveTime>,
    pub friday_end: Option<NaiveTime>,
    pub saturday_start: Option<NaiveTime>,
    pub saturday_end: Option<NaiveTime>,
    pub sunday_start: Option<NaiveTime>,
    pub sunday_end: Option<NaiveTime>,
    pub max_hours_per_week: Option<i32>,
    pub min_hours_per_week: Option<i32>,
    pub is_available_for_overtime: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftAssignment {
    pub id: i64,
    pub shift_id: i64,
    pub user_id: String,
    pub assigned_by: String,
    pub assignment_status: AssignmentStatus,
    pub acceptance_deadline: Option<NaiveDateTime>,
    pub response: Option<AssignmentResponse>,
    pub response_notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftAssignmentInput {
    pub shift_id: i64,
    pub user_id: String,
    pub assigned_by: String,
    pub acceptance_deadline: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}

impl std::fmt::Display for AssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentStatus::Pending => write!(f, "pending"),
            AssignmentStatus::Accepted => write!(f, "accepted"),
            AssignmentStatus::Declined => write!(f, "declined"),
            AssignmentStatus::Expired => write!(f, "expired"),
            AssignmentStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for AssignmentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(AssignmentStatus::Pending),
            "accepted" => Ok(AssignmentStatus::Accepted),
            "declined" => Ok(AssignmentStatus::Declined),
            "expired" => Ok(AssignmentStatus::Expired),
            "cancelled" => Ok(AssignmentStatus::Cancelled),
            _ => Err(format!("Invalid assignment status: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for AssignmentStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for AssignmentStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for AssignmentStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<AssignmentStatus>().map_err(|e| e.into())
    }
}

impl Default for AssignmentStatus {
    fn default() -> Self {
        AssignmentStatus::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentResponse {
    Accept,
    Decline,
}

impl std::fmt::Display for AssignmentResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentResponse::Accept => write!(f, "accept"),
            AssignmentResponse::Decline => write!(f, "decline"),
        }
    }
}

impl std::str::FromStr for AssignmentResponse {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "accept" => Ok(AssignmentResponse::Accept),
            "decline" => Ok(AssignmentResponse::Decline),
            _ => Err(format!("Invalid assignment response: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for AssignmentResponse {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for AssignmentResponse {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for AssignmentResponse {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<AssignmentResponse>().map_err(|e| e.into())
    }
}
