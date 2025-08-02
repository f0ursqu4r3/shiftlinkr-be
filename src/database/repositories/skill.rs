use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    models::{
        ProficiencyLevel, ShiftRequiredSkill, Skill, SkillInput, UserSkill, UserWithSkillResponse,
    },
    utils::sql,
};

#[derive(Clone)]
pub struct SkillRepository {
    pool: PgPool,
}

impl SkillRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_skill(&self, company_id: Uuid, input: SkillInput) -> Result<Skill> {
        let now = Utc::now().naive_utc();
        let skill = sqlx::query_as::<_, Skill>(&sql(r#"
            INSERT INTO
                skills (
                    company_id,
                    name,
                    description,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?)
            RETURNING
                id,
                company_id,
                name,
                description,
                created_at,
                updated_at
        "#))
        .bind(company_id)
        .bind(input.name)
        .bind(input.description)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(skill)
    }

    pub async fn find_by_id(&self, skill_id: Uuid, company_id: Uuid) -> Result<Option<Skill>> {
        let skill = sqlx::query_as::<_, Skill>(&sql(r#"
            SELECT
                id,
                company_id,
                name,
                description,
                created_at,
                updated_at
            FROM
                skills
            WHERE
                id = ?
                AND company_id = ?
        "#))
        .bind(skill_id)
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(skill)
    }

    pub async fn get_all_skills(&self, company_id: Uuid) -> Result<Vec<Skill>> {
        let skills = sqlx::query_as::<_, Skill>(&sql(r#"
            SELECT
                id,
                company_id,
                name,
                description,
                created_at,
                updated_at
            FROM
                skills
            WHERE
                company_id = ?
            ORDER BY
                name
        "#))
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(skills)
    }

    pub async fn update_skill(&self, id: Uuid, input: SkillInput) -> Result<Option<Skill>> {
        let now = Utc::now().naive_utc();
        let skill = sqlx::query_as::<_, Skill>(&sql(r#"
            UPDATE
                skills
            SET
                name = ?,
                description = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                company_id,
                name,
                description,
                created_at,
                updated_at
        "#))
        .bind(input.name)
        .bind(input.description)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(skill)
    }

    pub async fn delete_skill(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(&sql("DELETE FROM skills WHERE id = ?"))
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // User Skills
    pub async fn add_skill_to_user(
        &self,
        skill_id: Uuid,
        user_id: Uuid,
        proficiency_level: ProficiencyLevel,
    ) -> Result<UserSkill> {
        let now = Utc::now().naive_utc();
        let user_skill = sqlx::query_as::<_, UserSkill>(&sql(r#"
            INSERT INTO
                user_skills (
                    user_id,
                    skill_id,
                    proficiency_level,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?)
            RETURNING
                id,
                user_id,
                skill_id,
                proficiency_level,
                created_at,
                updated_at
        "#))
        .bind(user_id)
        .bind(skill_id)
        .bind(proficiency_level)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(user_skill)
    }

    pub async fn get_user_skills(&self, user_id: Uuid, company_id: Uuid) -> Result<Vec<UserSkill>> {
        let user_skills = sqlx::query_as::<_, UserSkill>(&sql(r#"
            SELECT
                id,
                user_id,
                skill_id,
                proficiency_level,
                created_at,
                updated_at
            FROM
                user_skills
            WHERE
                user_id = ?
                AND company_id = ?
        "#))
        .bind(user_id)
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(user_skills)
    }

    pub async fn update_user_skill(
        &self,
        user_id: Uuid,
        skill_id: Uuid,
        proficiency_level: ProficiencyLevel,
    ) -> Result<Option<UserSkill>> {
        let now = Utc::now().naive_utc();
        let user_skill = sqlx::query_as::<_, UserSkill>(&sql(r#"
            UPDATE
                user_skills
            SET
                proficiency_level = ?,
                updated_at = ?
            WHERE
                user_id = ?
                AND skill_id = ?
            RETURNING
                id,
                user_id,
                skill_id,
                proficiency_level,
                created_at,
                updated_at
        "#))
        .bind(proficiency_level.to_string())
        .bind(now)
        .bind(user_id)
        .bind(skill_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user_skill)
    }

    pub async fn remove_skill_from_user(
        &self,
        skill_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<()>> {
        let result = sqlx::query(&sql(r#"
            DELETE FROM
                user_skills
            WHERE
                user_id = ?
                AND skill_id = ?
        "#))
        .bind(user_id)
        .bind(skill_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    // Shift Required Skills
    pub async fn add_shift_required_skill(
        &self,
        shift_id: Uuid,
        skill_id: Uuid,
        required_level: ProficiencyLevel,
    ) -> Result<ShiftRequiredSkill> {
        let now = Utc::now().naive_utc();
        let shift_skill = sqlx::query_as::<_, ShiftRequiredSkill>(&sql(r#"
            INSERT INTO
                shift_required_skills (
                    shift_id,
                    skill_id,
                    required_level,
                    created_at
                )
            VALUES
                (?, ?, ?, ?)
            RETURNING
                id,
                shift_id,
                skill_id,
                required_level,
                created_at
        "#))
        .bind(shift_id)
        .bind(skill_id)
        .bind(required_level.to_string())
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(shift_skill)
    }

    pub async fn get_shift_required_skills(
        &self,
        shift_id: Uuid,
    ) -> Result<Vec<ShiftRequiredSkill>> {
        let shift_skills = sqlx::query_as::<_, ShiftRequiredSkill>(&sql(r#"
            SELECT
                id,
                shift_id,
                skill_id,
                required_level,
                created_at
            FROM
                shift_required_skills
            WHERE
                shift_id = ?
        "#))
        .bind(shift_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(shift_skills)
    }

    pub async fn remove_shift_required_skill(
        &self,
        shift_id: Uuid,
        skill_id: Uuid,
    ) -> Result<bool> {
        let result = sqlx::query(&sql(r#"
            DELETE FROM
                shift_required_skills
            WHERE
                shift_id = ?
                AND skill_id = ?
        "#))
        .bind(shift_id)
        .bind(skill_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_users_with_skill(
        &self,
        skill_id: Uuid,
        min_level: Option<ProficiencyLevel>,
    ) -> Result<Vec<UserWithSkillResponse>> {
        let base_query = r#"
            SELECT DISTINCT
                users.id,
                users.email,
                users.name,
                user_company.role,
                users.created_at,
                users.updated_at
                skills.id AS skill_id,
                user_skills.proficiency_level
            FROM
                user_skills
                JOIN skills ON skills.id = user_skills.skill_id
                LEFT JOIN users ON users.id = user_skills.user_id
                LEFT JOIN user_company ON user_company.user_id = users.id
                    AND user_company.company_id = skills.company_id
            WHERE
                skill_id = ?
        "#;
        let where_clause = if let Some(level) = min_level {
            match level {
                ProficiencyLevel::Expert => " AND proficiency_level = 'expert'",
                ProficiencyLevel::Advanced => " AND proficiency_level IN ('advanced', 'expert')",
                ProficiencyLevel::Intermediate => {
                    " AND proficiency_level IN ('intermediate', 'advanced', 'expert')"
                }
                ProficiencyLevel::Beginner => {
                    " AND proficiency_level IN ('beginner', 'intermediate', 'advanced', 'expert')"
                }
            }
        } else {
            ""
        };
        let query = format!("{}{}", base_query, where_clause);

        let user_ids = sqlx::query_as::<_, UserWithSkillResponse>(&sql(query.as_str()))
            .bind(skill_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(user_ids)
    }
}
