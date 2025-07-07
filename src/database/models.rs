use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Manager,
    Employee,
}

impl sqlx::Type<sqlx::Sqlite> for UserRole {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for UserRole {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        let s = match self {
            UserRole::Admin => "admin",
            UserRole::Manager => "manager",
            UserRole::Employee => "employee",
        };
        <&str as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for UserRole {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match s.as_str() {
            "admin" => Ok(UserRole::Admin),
            "manager" => Ok(UserRole::Manager),
            "employee" => Ok(UserRole::Employee),
            _ => Err(format!("Invalid UserRole: {}", s).into()),
        }
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Employee
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Manager => write!(f, "manager"),
            UserRole::Employee => write!(f, "employee"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "manager" => Ok(UserRole::Manager),
            "employee" => Ok(UserRole::Employee),
            _ => Err(format!("Invalid UserRole: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: Option<UserRole>,
}

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

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: UserRole,
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

// Invite link models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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
pub struct CreateInviteRequest {
    pub email: String,
    pub role: UserRole,
    pub team_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateInviteResponse {
    pub invite_link: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct GetInviteRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct GetInviteResponse {
    pub email: String,
    pub role: UserRole,
    pub team_name: Option<String>,
    pub inviter_name: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
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

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
        }
    }
}

impl User {
    pub fn new(email: String, password_hash: String, name: String, role: Option<UserRole>) -> Self {
        let now = Utc::now().naive_utc();
        User {
            id: Uuid::new_v4().to_string(),
            email,
            password_hash,
            name,
            role: role.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        }
    }
}

// Location models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Location {
    pub id: i64,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInput {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

// Team models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInput {
    pub name: String,
    pub description: Option<String>,
    pub location_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamMember {
    pub id: i64,
    pub team_id: i64,
    pub user_id: i64,
    pub created_at: NaiveDateTime,
}

// Shift models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Shift {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub team_id: Option<i64>,
    pub assigned_user_id: Option<i64>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub hourly_rate: Option<f64>,
    pub status: ShiftStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftInput {
    pub title: String,
    pub description: Option<String>,
    pub location_id: i64,
    pub team_id: Option<i64>,
    pub assigned_user_id: Option<i64>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub hourly_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShiftStatus {
    Open,
    Assigned,
    Completed,
    Cancelled,
}

impl std::fmt::Display for ShiftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftStatus::Open => write!(f, "open"),
            ShiftStatus::Assigned => write!(f, "assigned"),
            ShiftStatus::Completed => write!(f, "completed"),
            ShiftStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for ShiftStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(ShiftStatus::Open),
            "assigned" => Ok(ShiftStatus::Assigned),
            "completed" => Ok(ShiftStatus::Completed),
            "cancelled" => Ok(ShiftStatus::Cancelled),
            _ => Err(format!("Invalid shift status: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for ShiftStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ShiftStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ShiftStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<ShiftStatus>().map_err(|e| e.into())
    }
}

impl Default for ShiftStatus {
    fn default() -> Self {
        ShiftStatus::Open
    }
}
