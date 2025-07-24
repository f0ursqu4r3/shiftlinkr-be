use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserCompany {
    pub id: Uuid,
    pub user_id: String,
    pub company_id: Uuid,
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
#[serde(rename_all = "camelCase")]
pub struct CreateUserCompanyInput {
    pub user_id: String,
    pub company_id: String,
    pub pto_balance_hours: Option<i32>,
    pub sick_balance_hours: Option<i32>,
    pub personal_balance_hours: Option<i32>,
    pub pto_accrual_rate: Option<f32>,
    pub hire_date: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserCompanyInput {
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
        company_id: String,
        pto_balance_hours: i32,
        sick_balance_hours: i32,
        personal_balance_hours: i32,
        pto_accrual_rate: f32,
        hire_date: Option<NaiveDateTime>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            user_id,
            company_id: Uuid::parse_str(&company_id).expect("Invalid UUID format"),
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
