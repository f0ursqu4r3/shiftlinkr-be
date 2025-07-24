use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: Uuid,         // UUID primary key
    pub company_id: Uuid, // UUID for company references
    // Skill details
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserSkill {
    pub id: Uuid,       // UUID primary key
    pub user_id: Uuid,  // UUID for user references
    pub skill_id: Uuid, // UUID for skill references
    pub proficiency_level: ProficiencyLevel,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSkillInput {
    pub user_id: Uuid,  // UUID for user references
    pub skill_id: Uuid, // UUID for skill references
    pub proficiency_level: ProficiencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftRequiredSkill {
    pub id: Uuid,       // UUID primary key
    pub shift_id: Uuid, // UUID for shift references
    pub skill_id: Uuid, // UUID for skill references
    pub required_level: ProficiencyLevel,
    pub created_at: DateTime<Utc>, // TIMESTAMPTZ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftRequiredSkillInput {
    pub shift_id: Uuid, // UUID for shift references
    pub skill_id: Uuid, // UUID for skill references
    pub required_level: ProficiencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl std::fmt::Display for ProficiencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProficiencyLevel::Beginner => write!(f, "beginner"),
            ProficiencyLevel::Intermediate => write!(f, "intermediate"),
            ProficiencyLevel::Advanced => write!(f, "advanced"),
            ProficiencyLevel::Expert => write!(f, "expert"),
        }
    }
}

impl std::str::FromStr for ProficiencyLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "beginner" => Ok(ProficiencyLevel::Beginner),
            "intermediate" => Ok(ProficiencyLevel::Intermediate),
            "advanced" => Ok(ProficiencyLevel::Advanced),
            "expert" => Ok(ProficiencyLevel::Expert),
            _ => Err(format!("Invalid proficiency level: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ProficiencyLevel {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ProficiencyLevel {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ProficiencyLevel {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse::<ProficiencyLevel>().map_err(|e| e.into())
    }
}

impl Default for ProficiencyLevel {
    fn default() -> Self {
        ProficiencyLevel::Beginner
    }
}
