use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserCompany {
    pub id: Uuid,
    pub user_id: Uuid, // Fixed: should be UUID, not String
    pub company_id: Uuid,
    pub role: UserRole,               // Added: role field from schema
    pub is_primary: bool,             // Added: is_primary field
    pub hire_date: Option<NaiveDate>, // Fixed: DATE type should be NaiveDate
    pub pto_balance_hours: i32,
    pub sick_balance_hours: i32,
    pub personal_balance_hours: i32,
    pub pto_accrual_rate: BigDecimal, // Fixed: NUMERIC(5,2) should use BigDecimal
    pub last_accrual_date: Option<NaiveDate>, // Fixed: DATE type should be NaiveDate
    pub hourly_rate: Option<BigDecimal>, // Added: hourly_rate field
    pub overtime_rate_multiplier: Option<BigDecimal>, // Added: overtime_rate_multiplier field
    pub created_at: DateTime<Utc>,    // Fixed: TIMESTAMPTZ
    pub updated_at: DateTime<Utc>,    // Fixed: TIMESTAMPTZ
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum UserRole {
        Employee => "employee",
        Manager => "manager",
        Admin => "admin",
        Owner => "owner",
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Employee
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserCompanyInput {
    pub user_id: Uuid,                // Fixed: UUID type
    pub company_id: Uuid,             // Fixed: UUID type
    pub role: Option<UserRole>,       // Added: role field
    pub is_primary: Option<bool>,     // Added: is_primary field
    pub hire_date: Option<NaiveDate>, // Fixed: DATE type
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<BigDecimal>, // Fixed: BigDecimal type
    pub hourly_rate: Option<BigDecimal>,      // Added: hourly_rate field
    pub overtime_rate_multiplier: Option<BigDecimal>, // Added: overtime_rate_multiplier field
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserCompanyInput {
    pub role: Option<UserRole>,       // Added: role field
    pub is_primary: Option<bool>,     // Added: is_primary field
    pub hire_date: Option<NaiveDate>, // Fixed: DATE type
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<BigDecimal>, // Fixed: BigDecimal type
    pub last_accrual_date: Option<NaiveDate>, // Fixed: DATE type
    pub hourly_rate: Option<BigDecimal>,      // Added: hourly_rate field
    pub overtime_rate_multiplier: Option<BigDecimal>, // Added: overtime_rate_multiplier field
}

impl UserCompany {
    pub fn new(
        user_id: Uuid,
        company_id: Uuid,
        role: Option<UserRole>,
        is_primary: Option<bool>,
        hire_date: Option<NaiveDate>,
        pto_balance_hours: i32,
        sick_balance_hours: i32,
        personal_balance_hours: i32,
        pto_accrual_rate: BigDecimal,
        hourly_rate: Option<BigDecimal>,
        overtime_rate_multiplier: Option<BigDecimal>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            company_id,
            role: role.unwrap_or_default(),
            is_primary: is_primary.unwrap_or(false),
            hire_date,
            pto_balance_hours,
            sick_balance_hours,
            personal_balance_hours,
            pto_accrual_rate,
            last_accrual_date: None,
            hourly_rate,
            overtime_rate_multiplier: overtime_rate_multiplier
                .or_else(|| Some(BigDecimal::from_str("1.5").unwrap())),
            created_at: now,
            updated_at: now,
        }
    }
}
