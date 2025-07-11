use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserCompany {
    pub id: i64,
    pub user_id: String,
    pub company_id: i64,
    pub pto_balance_hours: i32,
    pub sick_balance_hours: i32,
    pub personal_balance_hours: i32,
    pub pto_accrual_rate: f32,
    pub hire_date: Option<NaiveDateTime>,
    pub last_accrual_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserCompanyRequest {
    pub user_id: String,
    pub company_id: i64,
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<f32>,
    pub hire_date: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserCompanyRequest {
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<f32>,
    pub hire_date: Option<NaiveDateTime>,
    pub last_accrual_date: Option<NaiveDateTime>,
}

impl UserCompany {
    pub fn new(
        user_id: String,
        company_id: i64,
        pto_balance_hours: i32,
        sick_balance_hours: i32,
        personal_balance_hours: i32,
        pto_accrual_rate: f32,
        hire_date: Option<NaiveDateTime>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: 0, // Will be set by database
            user_id,
            company_id,
            pto_balance_hours,
            sick_balance_hours,
            personal_balance_hours,
            pto_accrual_rate,
            hire_date,
            last_accrual_date: None,
            created_at: now,
            updated_at: now,
        }
    }
}
