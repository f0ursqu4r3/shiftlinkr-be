use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserShiftSchedule {
    pub id: Uuid,      // UUID primary key
    pub user_id: Uuid, // UUID for user references
    pub monday_start: Option<NaiveTime>,
    pub monday_end: Option<NaiveTime>,
    pub tuesday_start: Option<NaiveTime>,
    pub tuesday_end: Option<NaiveTime>,
    pub wednesday_start: Option<NaiveTime>,
    pub wednesday_end: Option<NaiveTime>,
    pub thursday_start: Option<NaiveTime>,
    pub thursday_end: Option<NaiveTime>,
    pub friday_start: Option<NaiveTime>,
    pub friday_end: Option<NaiveTime>,
    pub saturday_start: Option<NaiveTime>,
    pub saturday_end: Option<NaiveTime>,
    pub sunday_start: Option<NaiveTime>,
    pub sunday_end: Option<NaiveTime>,
    pub max_hours_per_week: Option<i32>,
    pub min_hours_per_week: Option<i32>,
    pub is_available_for_overtime: bool,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserShiftScheduleInput {
    pub user_id: Uuid, // UUID for user references
    pub monday_start: Option<NaiveTime>,
    pub monday_end: Option<NaiveTime>,
    pub tuesday_start: Option<NaiveTime>,
    pub tuesday_end: Option<NaiveTime>,
    pub wednesday_start: Option<NaiveTime>,
    pub wednesday_end: Option<NaiveTime>,
    pub thursday_start: Option<NaiveTime>,
    pub thursday_end: Option<NaiveTime>,
    pub friday_start: Option<NaiveTime>,
    pub friday_end: Option<NaiveTime>,
    pub saturday_start: Option<NaiveTime>,
    pub saturday_end: Option<NaiveTime>,
    pub sunday_start: Option<NaiveTime>,
    pub sunday_end: Option<NaiveTime>,
    pub max_hours_per_week: Option<i32>,
    pub min_hours_per_week: Option<i32>,
    pub is_available_for_overtime: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftAssignment {
    pub id: Uuid,          // UUID primary key
    pub shift_id: Uuid,    // UUID for shift references
    pub user_id: Uuid,     // UUID for user references
    pub assigned_by: Uuid, // UUID for user references
    pub assignment_status: AssignmentStatus,
    pub acceptance_deadline: Option<DateTime<Utc>>, // TIMESTAMPTZ
    pub response: Option<AssignmentResponse>,
    pub response_notes: Option<String>,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftAssignmentInput {
    pub shift_id: Uuid,                             // UUID for shift references
    pub user_id: Uuid,                              // UUID for user references
    pub assigned_by: Uuid,                          // UUID for user references
    pub acceptance_deadline: Option<DateTime<Utc>>, // TIMESTAMPTZ
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum AssignmentStatus {
        Pending => "pending",
        Accepted => "accepted",
        Declined => "declined",
        Expired => "expired",
        Cancelled => "cancelled",
    }
}

impl Default for AssignmentStatus {
    fn default() -> Self {
        AssignmentStatus::Pending
    }
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum AssignmentResponse {
        Accept => "accept",
        Decline => "decline",
    }
}
