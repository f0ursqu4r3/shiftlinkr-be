use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid, // UUID primary key
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum UserRole {
        Admin => "admin",
        Manager => "manager",
        Employee => "employee",
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Employee
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    /// User's email address
    pub email: String,
    /// User's password
    pub password: String,
    /// User's full name
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    /// User's unique identifier
    pub id: Uuid, // UUID type
    /// User's email address
    pub email: String,
    /// User's full name
    pub name: String,
}

impl User {
    pub fn new(email: String, password_hash: String, name: String) -> Self {
        Self {
            id: Uuid::new_v4(), // Generate UUID directly
            email,
            password_hash,
            name,
            created_at: Utc::now(), // Use DateTime<Utc>
            updated_at: Utc::now(),
        }
    }
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
        }
    }
}
