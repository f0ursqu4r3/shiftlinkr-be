use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalanceHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub balance_type: PtoBalanceType,
    pub change_type: PtoChangeType,
    pub hours_changed: i32,
    pub previous_balance: i32,
    pub new_balance: i32,
    pub description: Option<String>,
    pub related_time_off_id: Option<Uuid>, // Reference to time_off_requests table
    pub created_at: DateTime<Utc>,         // TIMESTAMPTZ
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum PtoBalanceType {
        Pto => "pto",
        Sick => "sick",
        Personal => "personal",
    }
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum PtoChangeType {
        Accrual => "accrual",
        Usage => "usage",
        Adjustment => "adjustment",
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalance {
    pub user_id: Uuid,
    pub pto_balance_hours: i32,
    pub sick_balance_hours: i32,
    pub personal_balance_hours: i32,
    pub pto_accrual_rate: BigDecimal, // DECIMAL type from PostgreSQL
    pub hire_date: Option<NaiveDate>, // DATE type
    pub last_accrual_date: Option<NaiveDate>, // DATE type
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalanceUpdateInput {
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<BigDecimal>, // DECIMAL type
    pub hire_date: Option<NaiveDate>,         // DATE type
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalanceAdjustmentInput {
    pub change_type: PtoChangeType,
    pub balance_type: PtoBalanceType,
    pub hours_changed: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtoBalanceAccrual {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub hire_date: Option<NaiveDate>,
    pub last_accrual_date: Option<NaiveDate>,
    pub months_since_last_accrual: i32,
    pub hours_to_accrue: i32,
    pub new_balance: i32,
}
