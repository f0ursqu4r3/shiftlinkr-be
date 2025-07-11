use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub role: UserRole,
    pub pto_balance_hours: i32,
    pub sick_balance_hours: i32,
    pub personal_balance_hours: i32,
    pub pto_accrual_rate: f32,
    pub hire_date: Option<NaiveDateTime>,
    pub last_accrual_date: Option<NaiveDateTime>,
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

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

impl User {
    pub fn new(email: String, password_hash: String, name: String, role: UserRole) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email,
            password_hash,
            name,
            role,
            pto_balance_hours: 0,
            sick_balance_hours: 0,
            personal_balance_hours: 0,
            pto_accrual_rate: 0.0,
            hire_date: None,
            last_accrual_date: None,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
        }
    }
}
