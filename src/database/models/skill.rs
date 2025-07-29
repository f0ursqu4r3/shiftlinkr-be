use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::string_enum;

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

string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ProficiencyLevel {
        Beginner => "beginner",
        Intermediate => "intermediate",
        Advanced => "advanced",
        Expert => "expert",
    }
}

impl Default for ProficiencyLevel {
    fn default() -> Self {
        ProficiencyLevel::Beginner
    }
}
