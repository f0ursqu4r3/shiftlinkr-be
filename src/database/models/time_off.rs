use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TimeOffRequest {
    pub id: i64,
    pub user_id: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub reason: Option<String>,
    pub request_type: TimeOffType,
    pub status: TimeOffStatus,
    pub approved_by: Option<String>,
    pub approval_notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeOffRequestInput {
    pub user_id: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub reason: String,
    pub request_type: TimeOffType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeOffType {
    Vacation,
    Sick,
    Personal,
    Emergency,
    Bereavement,
    MaternityPaternity,
    Other,
}

impl std::fmt::Display for TimeOffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeOffType::Vacation => write!(f, "vacation"),
            TimeOffType::Sick => write!(f, "sick"),
            TimeOffType::Personal => write!(f, "personal"),
            TimeOffType::Emergency => write!(f, "emergency"),
            TimeOffType::Bereavement => write!(f, "bereavement"),
            TimeOffType::MaternityPaternity => write!(f, "maternity_paternity"),
            TimeOffType::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for TimeOffType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vacation" => Ok(TimeOffType::Vacation),
            "sick" => Ok(TimeOffType::Sick),
            "personal" => Ok(TimeOffType::Personal),
            "emergency" => Ok(TimeOffType::Emergency),
            "bereavement" => Ok(TimeOffType::Bereavement),
            "maternity_paternity" => Ok(TimeOffType::MaternityPaternity),
            "other" => Ok(TimeOffType::Other),
            _ => Err(format!("Invalid time-off type: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for TimeOffType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for TimeOffType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TimeOffType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse::<TimeOffType>().map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeOffStatus {
    Pending,
    Approved,
    Denied,
    Cancelled,
}

impl std::fmt::Display for TimeOffStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeOffStatus::Pending => write!(f, "pending"),
            TimeOffStatus::Approved => write!(f, "approved"),
            TimeOffStatus::Denied => write!(f, "denied"),
            TimeOffStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for TimeOffStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TimeOffStatus::Pending),
            "approved" => Ok(TimeOffStatus::Approved),
            "denied" => Ok(TimeOffStatus::Denied),
            "cancelled" => Ok(TimeOffStatus::Cancelled),
            _ => Err(format!("Invalid time-off status: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for TimeOffStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for TimeOffStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TimeOffStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse::<TimeOffStatus>().map_err(|e| e.into())
    }
}
