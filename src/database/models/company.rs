use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub id: Uuid, // UUID primary key
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub logo_url: Option<String>,
    pub timezone: String,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ maps to DateTime<Utc>
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyEmployee {
    pub id: Uuid,         // UUID primary key
    pub user_id: Uuid,    // UUID type
    pub company_id: Uuid, // UUID type
    pub role: CompanyRole,
    pub is_primary: bool,
    pub hire_date: Option<NaiveDate>, // DATE maps to NaiveDate
    pub pto_balance_hours: i32,
    pub sick_balance_hours: i32,
    pub personal_balance_hours: i32,
    pub pto_accrual_rate: BigDecimal, // DECIMAL maps to BigDecimal
    pub last_accrual_date: Option<NaiveDate>, // DATE maps to NaiveDate
    pub created_at: DateTime<Utc>,    // TIMESTAMPTZ maps to DateTime<Utc>
    pub updated_at: DateTime<Utc>,
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum CompanyRole {
        Admin => "admin",
        Manager => "manager",
        Employee => "employee",
    }
}

impl Default for CompanyRole {
    fn default() -> Self {
        CompanyRole::Employee
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyInput {
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub logo_url: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub logo_url: Option<String>,
    pub timezone: String,
    pub role: CompanyRole,
    pub is_primary: bool,
    pub hire_date: Option<NaiveDate>, // DATE type for hire dates
    pub created_at: DateTime<Utc>,    // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>,    // TIMESTAMPTZ
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEmployeeToCompanyInput {
    pub user_id: Uuid, // UUID for PostgreSQL
    pub role: Option<CompanyRole>,
    pub is_primary: Option<bool>,
    pub hire_date: Option<NaiveDate>, // DATE type for hire dates
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyEmployeeInfo {
    pub id: Uuid, // UUID for user ID
    pub email: String,
    pub name: String,
    pub role: CompanyRole,
    pub is_primary: bool,
    pub hire_date: Option<NaiveDate>,      // DATE type
    pub created_at: Option<DateTime<Utc>>, // TIMESTAMPTZ
    pub updated_at: Option<DateTime<Utc>>, // TIMESTAMPTZ
}
