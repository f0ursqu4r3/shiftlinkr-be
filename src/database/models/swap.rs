use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwap {
    pub id: Uuid,
    pub requesting_user_id: Uuid,
    pub original_shift_id: Uuid,
    pub target_user_id: Option<Uuid>,
    pub target_shift_id: Option<Uuid>,
    pub notes: Option<String>,
    pub response: Option<String>,
    #[serde(rename = "type")]
    pub swap_type: ShiftSwapType,
    pub status: ShiftSwapStatus,
    pub actioned_by: Option<Uuid>,
    pub action_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwapInput {
    pub original_shift_id: Uuid,
    pub requesting_user_id: Uuid,
    pub target_user_id: Option<Uuid>,
    pub target_shift_id: Option<Uuid>,
    pub notes: Option<String>,
    pub swap_type: ShiftSwapType,
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum ShiftSwapType {
        Open => "open",     // Open to any qualified employee
        Targeted => "targeted", // Targeted to specific employee
    }
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum ShiftSwapStatus {
        Open => "open",
        Pending => "pending",
        Approved => "approved",
        Denied => "denied",
        Completed => "completed",
        Cancelled => "cancelled",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwapResponse {
    pub id: Uuid,
    pub swap_id: Option<Uuid>,
    pub responding_user_id: Uuid,
    pub status: ShiftSwapResponseStatus,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwapResponseInput {
    pub swap_id: String,
    pub responding_user_id: String,
    pub status: ShiftSwapResponseStatus,
    pub notes: Option<String>,
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum ShiftSwapResponseStatus {
        Pending => "pending",
        Interested => "interested",
        NotInterested => "not_interested",
        Accepted => "accepted",
        Declined => "declined",
    }
}
