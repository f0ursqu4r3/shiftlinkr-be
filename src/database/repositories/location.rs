use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::database::models::{Location, LocationInput, Team, TeamInput, TeamMember};
use crate::database::types::{TeamMemberRow, TeamRow};

pub struct LocationRepository {
    pool: SqlitePool,
}

impl LocationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_location(&self, input: LocationInput) -> Result<Location> {
        let now = Utc::now().naive_utc();
        let location = sqlx::query_as::<_, Location>(
            r#"
            INSERT INTO locations (name, address, phone, email, company_id, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING id, name, address, phone, email, company_id, created_at, updated_at
            "#,
        )
        .bind(&input.name)
        .bind(&input.address)
        .bind(&input.phone)
        .bind(&input.email)
        .bind(input.company_id)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn get_location_by_id(&self, id: i64) -> Result<Option<Location>> {
        let location = sqlx::query_as::<_, Location>(
            "SELECT id, name, address, phone, email, company_id, created_at, updated_at FROM locations WHERE id = ?1"
        )
        .bind(id)
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

    pub async fn get_locations_by_company(&self, company_id: i64) -> Result<Vec<Location>> {
        let locations = sqlx::query_as::<_, Location>(
            "SELECT id, name, address, phone, email, company_id, created_at, updated_at FROM locations WHERE company_id = ?1 ORDER BY name"
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(locations)
    }

    pub async fn update_location(&self, id: i64, input: LocationInput) -> Result<Option<Location>> {
        let now = Utc::now().naive_utc();
        let location = sqlx::query_as::<_, Location>(
            r#"
            UPDATE locations 
            SET name = ?1, address = ?2, phone = ?3, email = ?4, company_id = ?5, updated_at = ?6
            WHERE id = ?7
            RETURNING id, name, address, phone, email, company_id, created_at, updated_at
            "#,
        )
        .bind(&input.name)
        .bind(&input.address)
        .bind(&input.phone)
        .bind(&input.email)
        .bind(input.company_id)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(location)
    }

    pub async fn delete_location(&self, id: i64) -> Result<bool> {
        let result = sqlx::query!("DELETE FROM locations WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Team management methods
    pub async fn create_team(&self, input: TeamInput) -> Result<Team> {
        let now = Utc::now().naive_utc();
        let team = sqlx::query!(
            r#"
            INSERT INTO teams (name, description, location_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, name, description, location_id, created_at, updated_at
            "#,
            input.name,
            input.description,
            input.location_id,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Team {
            id: team.id.expect("Row ID should not be null"),
            name: team.name,
            description: team.description,
            location_id: team.location_id,
            created_at: team.created_at,
            updated_at: team.updated_at,
        })
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
        let rows = sqlx::query_as::<_, TeamRow>(
            "SELECT id, name, description, location_id, created_at, updated_at FROM teams WHERE location_id = ? ORDER BY name"
        )
        .bind(location_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
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
        let now = Utc::now().naive_utc();
        let row = sqlx::query_as::<_, TeamRow>(
            r#"
            UPDATE teams 
            SET name = ?, description = ?, location_id = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, name, description, location_id, created_at, updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.description)
        .bind(input.location_id)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn delete_team(&self, id: i64) -> Result<bool> {
        let result = sqlx::query!("DELETE FROM teams WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Team member management
    pub async fn add_team_member(&self, team_id: i64, user_id: i64) -> Result<TeamMember> {
        let now = Utc::now().naive_utc();
        let team_member = sqlx::query!(
            r#"
            INSERT INTO team_members (team_id, user_id, created_at)
            VALUES (?, ?, ?)
            RETURNING id, team_id, user_id, created_at
            "#,
            team_id,
            user_id,
            now,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TeamMember {
            id: team_member.id.expect("Row ID should not be null"),
            team_id: team_member.team_id,
            user_id: team_member.user_id,
            created_at: team_member.created_at,
        })
    }

    pub async fn get_team_members(&self, team_id: i64) -> Result<Vec<TeamMember>> {
        let rows = sqlx::query_as::<_, TeamMemberRow>(
            "SELECT id, team_id, user_id, created_at FROM team_members WHERE team_id = ?",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
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
        let teams = sqlx::query!(
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

        Ok(teams
            .into_iter()
            .map(|row| Team {
                id: row.id.expect("Row ID should not be null"),
                name: row.name,
                description: row.description,
                location_id: row.location_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }
}
