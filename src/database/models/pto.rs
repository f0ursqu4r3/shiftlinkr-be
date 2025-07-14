use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PtoBalanceHistory {
    pub id: i64,
    pub user_id: String,
    pub balance_type: PtoBalanceType,
    pub change_type: PtoChangeType,
    pub hours_changed: i32,
    pub previous_balance: i32,
    pub new_balance: i32,
    pub description: Option<String>,
    pub related_time_off_id: Option<i64>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PtoBalanceType {
    Pto,
    Sick,
    Personal,
}

impl sqlx::Type<sqlx::Sqlite> for PtoBalanceType {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for PtoBalanceType {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            PtoBalanceType::Pto => "pto",
            PtoBalanceType::Sick => "sick",
            PtoBalanceType::Personal => "personal",
        };
        <&str as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for PtoBalanceType {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
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

impl sqlx::Type<sqlx::Sqlite> for PtoChangeType {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for PtoChangeType {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            PtoChangeType::Accrual => "accrual",
            PtoChangeType::Usage => "usage",
            PtoChangeType::Adjustment => "adjustment",
        };
        <&str as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for PtoChangeType {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
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

#[derive(Debug, Serialize)]
pub struct PtoBalance {
    pub user_id: String,
    pub pto_balance_hours: i32,
    pub sick_balance_hours: i32,
    pub personal_balance_hours: i32,
    pub pto_accrual_rate: f32,
    pub hire_date: Option<NaiveDateTime>,
    pub last_accrual_date: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct PtoBalanceUpdate {
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<f32>,
    pub hire_date: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct PtoBalanceAdjustment {
    pub balance_type: PtoBalanceType,
    pub hours_changed: i32,
    pub description: String,
}
