use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyActivity {
    pub id: Uuid, // UUID primary key
    pub company_id: Uuid,
    pub user_id: Option<Uuid>,
    pub activity_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub description: String,
    pub metadata: Option<String>, // JSON as String in SQLite
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivityInput {
    pub company_id: Uuid,
    pub user_id: Option<Uuid>,
    pub activity_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub description: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub ip_address: String,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFilter {
    pub company_id: Uuid,
    pub activity_type: Option<String>,
    pub entity_type: Option<String>,
    pub user_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Common activity types for consistency
#[allow(non_snake_case)]
pub mod ActivityType {
    pub const USER_MANAGEMENT: &str = "user_management";
    pub const SHIFT_MANAGEMENT: &str = "shift_management";
    pub const LOCATION_MANAGEMENT: &str = "location_management";
    pub const TEAM_MANAGEMENT: &str = "team_management";
    pub const TIME_OFF_MANAGEMENT: &str = "time_off_management";
    pub const SCHEDULE_MANAGEMENT: &str = "schedule_management";
    pub const AUTHENTICATION: &str = "authentication";
    pub const SYSTEM: &str = "system";
    pub const SKILL_MANAGEMENT: &str = "skill_management";
    pub const SHIFT_SWAP: &str = "shift_swap";
}

// Common entity types
#[allow(non_snake_case)]
pub mod EntityType {
    pub const USER: &str = "user";
    pub const SHIFT: &str = "shift";
    pub const LOCATION: &str = "location";
    pub const TEAM: &str = "team";
    pub const TIME_OFF: &str = "time_off";
    pub const SHIFT_SWAP: &str = "shift_swap";
    pub const COMPANY: &str = "company";
    pub const SKILL: &str = "skill";
    pub const SCHEDULE: &str = "schedule";
}

// Common actions
#[allow(non_snake_case)]
pub mod Action {
    pub const CREATED: &str = "created";
    pub const UPDATED: &str = "updated";
    pub const DELETED: &str = "deleted";
    pub const LOGIN: &str = "login";
    pub const LOGOUT: &str = "logout";
    pub const INVITED: &str = "invited";
    pub const ACTIVATED: &str = "activated";
    pub const DEACTIVATED: &str = "deactivated";
    pub const ASSIGNED: &str = "assigned";
    pub const UNASSIGNED: &str = "unassigned";
    pub const APPROVED: &str = "approved";
    pub const REJECTED: &str = "rejected";
    pub const CLAIMED: &str = "claimed";
    pub const RELEASED: &str = "released";
    pub const CANCELLED: &str = "cancelled";
    pub const SWITCH_COMPANY: &str = "switch_company";
    pub const ACCEPTED: &str = "accepted";
    pub const DECLINED: &str = "declined";
    pub const MEMBER_ADDED: &str = "member_added";
    pub const MEMBER_REMOVED: &str = "member_removed";
    pub const SKILL_ADDED: &str = "skill_added";
    pub const SKILL_REMOVED: &str = "skill_removed";
    pub const SKILL_UPDATED: &str = "skill_updated";
    pub const SKILL_ASSIGNED: &str = "skill_assigned";
    pub const SKILL_UNASSIGNED: &str = "skill_unassigned";
}
