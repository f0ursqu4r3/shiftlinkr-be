use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::user::{UserInfo, UserRole};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InviteToken {
    pub id: String,
    pub email: String,
    pub token: String,
    pub inviter_id: String,
    pub role: UserRole,
    pub team_id: Option<i64>,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: UserRole,
    pub team_id: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteResponse {
    pub invite_link: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInviteRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInviteResponse {
    pub email: String,
    pub role: UserRole,
    pub team_name: Option<String>,
    pub inviter_name: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInviteRequest {
    pub token: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AcceptInviteResponse {
    pub token: String,
    pub user: UserInfo,
}
