use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub logo_url: Option<String>,
    pub timezone: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CompanyEmployee {
    pub id: i64,
    pub user_id: String,
    pub company_id: i64,
    pub role: CompanyRole,
    pub is_primary: bool,
    pub hired_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CompanyRole {
    Employee,
    Manager,
    Admin,
}

impl sqlx::Type<sqlx::Sqlite> for CompanyRole {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for CompanyRole {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        let s = match self {
            CompanyRole::Admin => "admin",
            CompanyRole::Manager => "manager",
            CompanyRole::Employee => "employee",
        };
        <&str as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for CompanyRole {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match s.as_str() {
            "admin" => Ok(CompanyRole::Admin),
            "manager" => Ok(CompanyRole::Manager),
            "employee" => Ok(CompanyRole::Employee),
            _ => Err(format!("Invalid CompanyRole: {}", s).into()),
        }
    }
}

impl Default for CompanyRole {
    fn default() -> Self {
        CompanyRole::Employee
    }
}

impl std::fmt::Display for CompanyRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompanyRole::Admin => write!(f, "admin"),
            CompanyRole::Manager => write!(f, "manager"),
            CompanyRole::Employee => write!(f, "employee"),
        }
    }
}

impl std::str::FromStr for CompanyRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(CompanyRole::Admin),
            "manager" => Ok(CompanyRole::Manager),
            "employee" => Ok(CompanyRole::Employee),
            _ => Err(format!("Invalid CompanyRole: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCompanyRequest {
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub logo_url: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompanyInfo {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub logo_url: Option<String>,
    pub timezone: String,
    pub role: CompanyRole,
    pub is_primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddEmployeeToCompanyRequest {
    pub user_id: String,
    pub role: CompanyRole,
    pub is_primary: Option<bool>,
    pub hired_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct CompanyEmployeeInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: CompanyRole,
    pub is_primary: bool,
    pub hired_at: Option<NaiveDateTime>,
}
