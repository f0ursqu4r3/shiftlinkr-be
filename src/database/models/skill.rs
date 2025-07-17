use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub id: i64,
    pub user_id: String,
    pub skill_id: i64,
    pub proficiency_level: ProficiencyLevel,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSkillInput {
    pub user_id: String,
    pub skill_id: i64,
    pub proficiency_level: ProficiencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShiftRequiredSkill {
    pub id: i64,
    pub shift_id: i64,
    pub skill_id: i64,
    pub required_level: ProficiencyLevel,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftRequiredSkillInput {
    pub shift_id: i64,
    pub skill_id: i64,
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

impl sqlx::Type<sqlx::Sqlite> for ProficiencyLevel {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ProficiencyLevel {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&s, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ProficiencyLevel {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        s.parse::<ProficiencyLevel>().map_err(|e| e.into())
    }
}

impl Default for ProficiencyLevel {
    fn default() -> Self {
        ProficiencyLevel::Beginner
    }
}
