use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::CompanyInfo;

use super::user::User;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    /// User's email address
    pub email: String,
    /// User's password
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// JWT authentication token
    pub token: String,
    /// User information
    pub user: User,
    // Company information if available
    pub company: Option<CompanyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetToken {
    pub id: Uuid,      // UUID primary key
    pub user_id: Uuid, // UUID foreign key
    pub token: String,
    pub expires_at: DateTime<Utc>,      // TIMESTAMPTZ
    pub used_at: Option<DateTime<Utc>>, // TIMESTAMPTZ
    pub created_at: DateTime<Utc>,      // TIMESTAMPTZ
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordInput {
    /// Email address to send password reset to
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordInput {
    /// Password reset token from email
    pub token: String,
    /// New password
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}
