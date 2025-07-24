use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalanceHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance_type: PtoBalanceType,
    pub change_type: PtoChangeType,
    pub hours_changed: i32,
    pub previous_balance: i32,
    pub new_balance: i32,
    pub description: Option<String>,
    pub related_time_off_id: Option<Uuid>, // Reference to time_off_requests table
    pub created_at: DateTime<Utc>,         // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PtoBalanceType {
    Pto,
    Sick,
    Personal,
}

impl sqlx::Type<sqlx::Postgres> for PtoBalanceType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for PtoBalanceType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            PtoBalanceType::Pto => "pto",
            PtoBalanceType::Sick => "sick",
            PtoBalanceType::Personal => "personal",
        };
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for PtoBalanceType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "pto" => Ok(PtoBalanceType::Pto),
            "sick" => Ok(PtoBalanceType::Sick),
            "personal" => Ok(PtoBalanceType::Personal),
            _ => Err(format!("Invalid PtoBalanceType: {}", s).into()),
        }
    }
}

impl std::fmt::Display for PtoBalanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtoBalanceType::Pto => write!(f, "pto"),
            PtoBalanceType::Sick => write!(f, "sick"),
            PtoBalanceType::Personal => write!(f, "personal"),
        }
    }
}

impl std::str::FromStr for PtoBalanceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pto" => Ok(PtoBalanceType::Pto),
            "sick" => Ok(PtoBalanceType::Sick),
            "personal" => Ok(PtoBalanceType::Personal),
            _ => Err(format!("Invalid PtoBalanceType: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PtoChangeType {
    Accrual,
    Usage,
    Adjustment,
}

impl sqlx::Type<sqlx::Postgres> for PtoChangeType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for PtoChangeType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            PtoChangeType::Accrual => "accrual",
            PtoChangeType::Usage => "usage",
            PtoChangeType::Adjustment => "adjustment",
        };
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for PtoChangeType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "accrual" => Ok(PtoChangeType::Accrual),
            "usage" => Ok(PtoChangeType::Usage),
            "adjustment" => Ok(PtoChangeType::Adjustment),
            _ => Err(format!("Invalid PtoChangeType: {}", s).into()),
        }
    }
}

impl std::fmt::Display for PtoChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtoChangeType::Accrual => write!(f, "accrual"),
            PtoChangeType::Usage => write!(f, "usage"),
            PtoChangeType::Adjustment => write!(f, "adjustment"),
        }
    }
}

impl std::str::FromStr for PtoChangeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "accrual" => Ok(PtoChangeType::Accrual),
            "usage" => Ok(PtoChangeType::Usage),
            "adjustment" => Ok(PtoChangeType::Adjustment),
            _ => Err(format!("Invalid PtoChangeType: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalance {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub pto_balance_hours: i32,
    pub sick_balance_hours: i32,
    pub personal_balance_hours: i32,
    pub pto_accrual_rate: BigDecimal, // DECIMAL type from PostgreSQL
    pub hire_date: Option<NaiveDate>, // DATE type
    pub last_accrual_date: Option<NaiveDate>, // DATE type
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalanceUpdateInput {
    pub company_id: Uuid,
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<BigDecimal>, // DECIMAL type
    pub hire_date: Option<NaiveDate>,         // DATE type
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalanceAdjustmentInput {
    pub company_id: Uuid, // UUID for company references
    pub balance_type: PtoBalanceType,
    pub hours_changed: i32,
    pub description: String,
}
