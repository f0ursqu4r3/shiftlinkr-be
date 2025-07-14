use anyhow::Result;
use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::database::models::{
    ProficiencyLevel, ShiftRequiredSkill, ShiftRequiredSkillInput, Skill, SkillInput, UserSkill,
    UserSkillInput,
};

pub struct SkillRepository {
    pool: SqlitePool,
}

impl SkillRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_skill(&self, input: SkillInput) -> Result<Skill> {
        let now = Utc::now().naive_utc();
        let skill = sqlx::query_as::<_, Skill>(
            r#"
            INSERT INTO skills (name, description, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            RETURNING id, name, description, created_at, updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.description)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(skill)
    }

    pub async fn get_skill_by_id(&self, id: i64) -> Result<Option<Skill>> {
        let skill = sqlx::query_as::<_, Skill>(
            "SELECT id, name, description, created_at, updated_at FROM skills WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(skill)
    }

    pub async fn get_all_skills(&self) -> Result<Vec<Skill>> {
        let skills = sqlx::query_as::<_, Skill>(
            "SELECT id, name, description, created_at, updated_at FROM skills ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(skills)
    }

    pub async fn update_skill(&self, id: i64, input: SkillInput) -> Result<Option<Skill>> {
        let now = Utc::now().naive_utc();
        let skill = sqlx::query_as::<_, Skill>(
            r#"
            UPDATE skills SET name = ?, description = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, name, description, created_at, updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.description)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(skill)
    }

    pub async fn delete_skill(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM skills WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // User Skills
    pub async fn add_user_skill(&self, input: UserSkillInput) -> Result<UserSkill> {
        let now = Utc::now().naive_utc();
        let user_skill = sqlx::query_as::<_, UserSkill>(
            r#"
            INSERT INTO user_skills (user_id, skill_id, proficiency_level, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, user_id, skill_id, proficiency_level, created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.skill_id)
        .bind(input.proficiency_level.to_string())
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(user_skill)
    }

    pub async fn get_user_skills(&self, user_id: &str) -> Result<Vec<UserSkill>> {
        let user_skills = sqlx::query_as::<_, UserSkill>(
            "SELECT id, user_id, skill_id, proficiency_level, created_at, updated_at FROM user_skills WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(user_skills)
    }

    pub async fn update_user_skill(
        &self,
        id: i64,
        proficiency_level: ProficiencyLevel,
    ) -> Result<Option<UserSkill>> {
        let now = Utc::now().naive_utc();
        let user_skill = sqlx::query_as::<_, UserSkill>(
            r#"
            UPDATE user_skills SET proficiency_level = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, user_id, skill_id, proficiency_level, created_at, updated_at
            "#,
        )
        .bind(proficiency_level.to_string())
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user_skill)
    }

    pub async fn remove_user_skill(&self, user_id: &str, skill_id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM user_skills WHERE user_id = ? AND skill_id = ?")
            .bind(user_id)
            .bind(skill_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Shift Required Skills
    pub async fn add_shift_required_skill(
        &self,
        input: ShiftRequiredSkillInput,
    ) -> Result<ShiftRequiredSkill> {
        let now = Utc::now().naive_utc();
        let shift_skill = sqlx::query_as::<_, ShiftRequiredSkill>(
            r#"
            INSERT INTO shift_required_skills (shift_id, skill_id, required_level, created_at)
            VALUES (?, ?, ?, ?)
            RETURNING id, shift_id, skill_id, required_level, created_at
            "#,
        )
        .bind(input.shift_id)
        .bind(input.skill_id)
        .bind(input.required_level.to_string())
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(shift_skill)
    }

    pub async fn get_shift_required_skills(
        &self,
        shift_id: i64,
    ) -> Result<Vec<ShiftRequiredSkill>> {
        let shift_skills = sqlx::query_as::<_, ShiftRequiredSkill>(
            "SELECT id, shift_id, skill_id, required_level, created_at FROM shift_required_skills WHERE shift_id = ?"
        )
        .bind(shift_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(shift_skills)
    }

    pub async fn remove_shift_required_skill(&self, shift_id: i64, skill_id: i64) -> Result<bool> {
        let result =
            sqlx::query("DELETE FROM shift_required_skills WHERE shift_id = ? AND skill_id = ?")
                .bind(shift_id)
                .bind(skill_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_users_with_skill(
        &self,
        skill_id: i64,
        min_level: Option<ProficiencyLevel>,
    ) -> Result<Vec<String>> {
        let query = if let Some(level) = min_level {
            match level {
                ProficiencyLevel::Expert => {
                    "SELECT DISTINCT user_id FROM user_skills WHERE skill_id = ? AND proficiency_level = 'expert'"
                },
                ProficiencyLevel::Advanced => {
                    "SELECT DISTINCT user_id FROM user_skills WHERE skill_id = ? AND proficiency_level IN ('advanced', 'expert')"
                },
                ProficiencyLevel::Intermediate => {
                    "SELECT DISTINCT user_id FROM user_skills WHERE skill_id = ? AND proficiency_level IN ('intermediate', 'advanced', 'expert')"
                },
                ProficiencyLevel::Beginner => {
                    "SELECT DISTINCT user_id FROM user_skills WHERE skill_id = ? AND proficiency_level IN ('beginner', 'intermediate', 'advanced', 'expert')"
                }
            }
        } else {
            "SELECT DISTINCT user_id FROM user_skills WHERE skill_id = ?"
        };

        let rows = sqlx::query(query)
            .bind(skill_id)
            .fetch_all(&self.pool)
            .await?;

        let user_ids = rows
            .into_iter()
            .map(|row| row.get::<String, _>("user_id"))
            .collect();

        Ok(user_ids)
    }
}
