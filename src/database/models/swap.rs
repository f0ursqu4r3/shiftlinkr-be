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
    pub swap_id: Uuid, // Fixed: should be UUID, not Option<Uuid>
    pub responding_user_id: Uuid,
    pub response_type: ShiftSwapResponseType, // Fixed: use enum instead of String for type safety
    pub notes: Option<String>,                // Fixed: should be Option<String>
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftSwapResponseInput {
    pub swap_id: Uuid,                        // Fixed: should be UUID, not String
    pub responding_user_id: Uuid,             // Fixed: should be UUID, not String
    pub response_type: ShiftSwapResponseType, // Fixed: use enum for type safety
    pub notes: Option<String>,
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum ShiftSwapResponseType {
        Interested => "interested",
        Accepted => "accepted",
        Declined => "declined",
    }
}

impl Default for ShiftSwapResponseType {
    fn default() -> Self {
        ShiftSwapResponseType::Interested
    }
}
