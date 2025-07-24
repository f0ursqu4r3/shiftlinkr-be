use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::{Location, LocationInput, Team, TeamInput, TeamMember};

pub struct LocationRepository {
    pool: PgPool,
}

impl LocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_location(&self, input: LocationInput) -> Result<Location> {
        let now = Utc::now();
        let location = sqlx::query_as::<_, Location>(
            r#"
            INSERT INTO
                locations (
                    name,
                    address,
                    phone,
                    email,
                    company_id,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id,
                name,
                address,
                phone,
                email,
                company_id,
                created_at,
                updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.address)
        .bind(input.phone)
        .bind(input.email)
        .bind(input.company_id)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Location>> {
        let location = sqlx::query_as::<_, Location>(
            "SELECT id, name, address, phone, email, company_id, created_at, updated_at FROM locations WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn find_by_team_id(&self, team_id: Uuid) -> Result<Option<Location>> {
        let location = sqlx::query_as::<_, Location>(
            "SELECT l.id, l.name, l.address, l.phone, l.email, l.company_id, l.created_at, l.updated_at FROM locations l INNER JOIN teams t ON l.id = t.location_id WHERE t.id = ?"
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn get_all_locations(&self) -> Result<Vec<Location>> {
        let locations = sqlx::query_as::<_, Location>(
            "SELECT id, name, address, phone, email, company_id, created_at, updated_at FROM locations ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(locations)
    }

    pub async fn get_locations_by_company(&self, company_id: Uuid) -> Result<Vec<Location>> {
        let locations = sqlx::query_as::<_, Location>(
            "SELECT id, name, address, phone, email, company_id, created_at, updated_at FROM locations WHERE company_id = ? ORDER BY name"
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(locations)
    }

    pub async fn update_location(
        &self,
        id: Uuid,
        input: LocationInput,
    ) -> Result<Option<Location>> {
        let now = Utc::now();
        let location = sqlx::query_as::<_, Location>(
            r#"
            UPDATE
                locations
            SET
                name = ?,
                address = ?,
                phone = ?,
                email = ?,
                company_id = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                name,
                address,
                phone,
                email,
                company_id,
                created_at,
                updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.address)
        .bind(input.phone)
        .bind(input.email)
        .bind(input.company_id)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn delete_location(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM locations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Team management methods
    pub async fn create_team(&self, input: TeamInput) -> Result<Team> {
        let now = Utc::now();
        let team = sqlx::query_as::<_, Team>(
            r#"
            INSERT INTO
                teams (
                    name,
                    description,
                    location_id,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?)
            RETURNING
                id,
                name,
                description,
                location_id,
                created_at,
                updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.description)
        .bind(input.location_id)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(team)
    }

    pub async fn get_team_by_id(&self, id: Uuid) -> Result<Option<Team>> {
        let team = sqlx::query_as::<_, Team>(
            "SELECT id, name, description, location_id, created_at, updated_at FROM teams WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(team)
    }

    pub async fn get_teams_by_location(&self, location_id: Uuid) -> Result<Vec<Team>> {
        let teams = sqlx::query_as::<_, Team>(
            "SELECT id, name, description, location_id, created_at, updated_at FROM teams WHERE location_id = ? ORDER BY name",
        )
        .bind(location_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    pub async fn get_all_teams_for_company(&self, company_id: Uuid) -> Result<Vec<Team>> {
        let teams = sqlx::query_as::<_, Team>(
            "SELECT id, name, description, location_id, created_at, updated_at FROM teams WHERE company_id = ? ORDER BY name"
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    pub async fn update_team(&self, id: Uuid, input: TeamInput) -> Result<Option<Team>> {
        let now = Utc::now();
        let team = sqlx::query_as::<_, Team>(
            r#"
            UPDATE
                teams 
            SET
                name = ?,
                description = ?,
                location_id = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                name,
                description,
                location_id,
                created_at,
                updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.description)
        .bind(input.location_id)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(team)
    }

    pub async fn delete_team(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM teams WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Team member management
    pub async fn add_team_member(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMember> {
        let now = Utc::now();
        let team_member = sqlx::query_as::<_, TeamMember>(
            r#"
            INSERT INTO
                team_members (
                    team_id,
                    user_id,
                    created_at
                )
            VALUES
                (?, ?, ?)
            RETURNING
                id,
                team_id,
                user_id,
                created_at
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(team_member)
    }

    pub async fn get_team_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>> {
        let team_members = sqlx::query_as::<_, TeamMember>(
            "SELECT id, team_id, user_id, created_at FROM team_members WHERE team_id = ?",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(team_members)
    }

    pub async fn remove_team_member(&self, team_id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM team_members WHERE team_id = ? AND user_id = ?")
            .bind(team_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_user_teams(&self, user_id: Uuid) -> Result<Vec<Team>> {
        let teams = sqlx::query_as::<_, Team>(
            r#"
            SELECT
                t.id,
                t.name,
                t.description,
                t.location_id,
                t.created_at,
                t.updated_at
            FROM
                teams t
                INNER JOIN team_members tm ON t.id = tm.team_id
            WHERE
                tm.user_id = ?
            ORDER BY
                t.name
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }
}
