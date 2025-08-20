use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WageHistory {
    pub id: Uuid,                                     // UUID primary key
    pub user_id: Uuid,                                // UUID for user references
    pub company_id: Uuid,                             // UUID for company references
    pub hourly_rate: BigDecimal,                      // NUMERIC(10,2)
    pub overtime_rate_multiplier: Option<BigDecimal>, // NUMERIC(3,2)
    pub effective_date: NaiveDate,                    // DATE
    pub end_date: Option<NaiveDate>,                  // DATE
    pub changed_by: Option<Uuid>,                     // UUID for user references
    pub change_reason: Option<String>,                // TEXT
    pub created_at: DateTime<Utc>,                    // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWageHistoryInput {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub hourly_rate: BigDecimal,
    pub overtime_rate_multiplier: Option<BigDecimal>,
    pub effective_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub changed_by: Option<Uuid>,
    pub change_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWageHistoryInput {
    pub hourly_rate: Option<BigDecimal>,
    pub overtime_rate_multiplier: Option<BigDecimal>,
    pub effective_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub changed_by: Option<Uuid>,
    pub change_reason: Option<String>,
}
