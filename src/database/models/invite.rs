use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::User;

use super::company::CompanyRole;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InviteToken {
    pub id: Uuid, // UUID primary key
    pub email: String,
    pub token: String,
    pub inviter_id: Uuid, // UUID for user references
    pub role: CompanyRole,
    pub company_id: Uuid,               // UUID for company references
    pub team_id: Option<Uuid>,          // UUID for team references
    pub expires_at: DateTime<Utc>,      // TIMESTAMPTZ
    pub used_at: Option<DateTime<Utc>>, // TIMESTAMPTZ
    pub created_at: DateTime<Utc>,      // TIMESTAMPTZ
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteInput {
    pub email: String,
    pub role: CompanyRole,
    pub team_id: Option<Uuid>, // UUID type
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteResponse {
    pub invite_link: String,
    pub expires_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInviteInput {
    pub token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInviteResponse {
    pub email: String,
    pub role: CompanyRole,
    pub team_id: Option<Uuid>, // UUID type
    pub company_id: Uuid,      // UUID type
    pub company_name: String,
    pub inviter_name: String,
    pub expires_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInviteInput {
    pub token: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AcceptInviteResponse {
    pub token: String,
    pub user: User,
}
