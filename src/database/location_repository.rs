use sqlx::SqlitePool;
use anyhow::Result;
use chrono::{DateTime, Utc};

use super::models::{Location, LocationInput, Team, TeamInput, TeamMember};

pub struct LocationRepository {
    pool: SqlitePool,
}

impl LocationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_location(&self, input: LocationInput) -> Result<Location> {
        let location = sqlx::query_as!(
            Location,
            r#"
            INSERT INTO locations (name, address, phone, email, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, name, address, phone, email, created_at, updated_at
            "#,
            input.name,
            input.address,
            input.phone,
            input.email,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn get_location_by_id(&self, id: i64) -> Result<Option<Location>> {
        let location = sqlx::query_as!(
            Location,
            "SELECT id, name, address, phone, email, created_at, updated_at FROM locations WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn get_all_locations(&self) -> Result<Vec<Location>> {
        let locations = sqlx::query_as!(
            Location,
            "SELECT id, name, address, phone, email, created_at, updated_at FROM locations ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(locations)
    }

    pub async fn update_location(&self, id: i64, input: LocationInput) -> Result<Option<Location>> {
        let location = sqlx::query_as!(
            Location,
            r#"
            UPDATE locations 
            SET name = ?, address = ?, phone = ?, email = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, name, address, phone, email, created_at, updated_at
            "#,
            input.name,
            input.address,
            input.phone,
            input.email,
            Utc::now(),
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn delete_location(&self, id: i64) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM locations WHERE id = ?",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // Team management methods
    pub async fn create_team(&self, input: TeamInput) -> Result<Team> {
        let team = sqlx::query_as!(
            Team,
            r#"
            INSERT INTO teams (name, description, location_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, name, description, location_id, created_at, updated_at
            "#,
            input.name,
            input.description,
            input.location_id,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(team)
    }

    pub async fn get_team_by_id(&self, id: i64) -> Result<Option<Team>> {
        let team = sqlx::query_as!(
            Team,
            "SELECT id, name, description, location_id, created_at, updated_at FROM teams WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(team)
    }

    pub async fn get_teams_by_location(&self, location_id: i64) -> Result<Vec<Team>> {
        let teams = sqlx::query_as!(
            Team,
            "SELECT id, name, description, location_id, created_at, updated_at FROM teams WHERE location_id = ? ORDER BY name",
            location_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    pub async fn get_all_teams(&self) -> Result<Vec<Team>> {
        let teams = sqlx::query_as!(
            Team,
            "SELECT id, name, description, location_id, created_at, updated_at FROM teams ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    pub async fn update_team(&self, id: i64, input: TeamInput) -> Result<Option<Team>> {
        let team = sqlx::query_as!(
            Team,
            r#"
            UPDATE teams 
            SET name = ?, description = ?, location_id = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, name, description, location_id, created_at, updated_at
            "#,
            input.name,
            input.description,
            input.location_id,
            Utc::now(),
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(team)
    }

    pub async fn delete_team(&self, id: i64) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM teams WHERE id = ?",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // Team member management
    pub async fn add_team_member(&self, team_id: i64, user_id: i64) -> Result<TeamMember> {
        let team_member = sqlx::query_as!(
            TeamMember,
            r#"
            INSERT INTO team_members (team_id, user_id, created_at)
            VALUES (?, ?, ?)
            RETURNING id, team_id, user_id, created_at
            "#,
            team_id,
            user_id,
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(team_member)
    }

    pub async fn get_team_members(&self, team_id: i64) -> Result<Vec<TeamMember>> {
        let members = sqlx::query_as!(
            TeamMember,
            "SELECT id, team_id, user_id, created_at FROM team_members WHERE team_id = ?",
            team_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    pub async fn remove_team_member(&self, team_id: i64, user_id: i64) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM team_members WHERE team_id = ? AND user_id = ?",
            team_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_user_teams(&self, user_id: i64) -> Result<Vec<Team>> {
        let teams = sqlx::query_as!(
            Team,
            r#"
            SELECT t.id, t.name, t.description, t.location_id, t.created_at, t.updated_at
            FROM teams t
            INNER JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = ?
            ORDER BY t.name
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }
}
