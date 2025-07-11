use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::user::UserInfo;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasswordResetToken {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}
