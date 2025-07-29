use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TimeOffRequest {
    pub id: Uuid,                  // UUID primary key
    pub user_id: Uuid,             // UUID for user references
    pub start_date: DateTime<Utc>, // TIMESTAMPTZ
    pub end_date: DateTime<Utc>,   // TIMESTAMPTZ
    pub reason: Option<String>,
    pub request_type: TimeOffType,
    pub status: TimeOffStatus,
    pub approved_by: Option<Uuid>, // UUID for user references
    pub approval_notes: Option<String>,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeOffRequestInput {
    pub user_id: Uuid,             // UUID for user references
    pub start_date: DateTime<Utc>, // DATE type
    pub end_date: DateTime<Utc>,   // DATE type
    pub reason: String,
    pub request_type: TimeOffType,
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum TimeOffType {
        Vacation => "vacation",
        Sick => "sick",
        Personal => "personal",
        Emergency => "emergency",
        Bereavement => "bereavement",
        MaternityPaternity => "maternity_paternity",
        Other => "other",
    }
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum TimeOffStatus {
        Pending => "pending",
        Approved => "approved",
        Denied => "denied",
        Cancelled => "cancelled",
    }
}
