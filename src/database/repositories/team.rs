use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::{
    get_pool,
    models::{Team, TeamInput, TeamMember},
    utils::sql,
};

// Team management methods
pub async fn create_team(input: TeamInput) -> Result<Team> {
    let now = Utc::now();
    let team = sqlx::query_as::<_, Team>(&sql(r#"
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
        "#))
    .bind(input.name)
    .bind(input.description)
    .bind(input.location_id)
    .bind(now)
    .bind(now)
    .fetch_one(get_pool())
    .await?;

    Ok(team)
}

pub async fn get_team_by_id(id: Uuid) -> Result<Option<Team>> {
    let team = sqlx::query_as::<_, Team>(&sql(r#"
            SELECT
                id,
                name,
                description,
                location_id,
                created_at,
                updated_at
            FROM
                teams
            WHERE
                id = ?
        "#))
    .bind(id)
    .fetch_optional(get_pool())
    .await?;

    Ok(team)
}

pub async fn get_teams_by_location(location_id: Uuid) -> Result<Vec<Team>> {
    let teams = sqlx::query_as::<_, Team>(&sql(r#"
            SELECT
                id,
                name,
                description,
                location_id,
                created_at,
                updated_at
            FROM
                teams
            WHERE
                location_id = ?
            ORDER BY
                name
        "#))
    .bind(location_id)
    .fetch_all(get_pool())
    .await?;

    Ok(teams)
}

pub async fn get_all_teams_for_company(company_id: Uuid) -> Result<Vec<Team>> {
    let teams = sqlx::query_as::<_, Team>(&sql(r#"
            SELECT
                t.id,
                t.name,
                t.description,
                t.location_id,
                t.created_at,
                t.updated_at
            FROM
                teams t
                INNER JOIN locations l ON t.location_id = l.id
            WHERE
                l.company_id = ?
            ORDER BY
                t.name
        "#))
    .bind(company_id)
    .fetch_all(get_pool())
    .await?;

    Ok(teams)
}

pub async fn update_team(id: Uuid, input: TeamInput) -> Result<Option<Team>> {
    let now = Utc::now();
    let team = sqlx::query_as::<_, Team>(&sql(r#"
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
        "#))
    .bind(input.name)
    .bind(input.description)
    .bind(input.location_id)
    .bind(now)
    .bind(id)
    .fetch_optional(get_pool())
    .await?;

    Ok(team)
}

pub async fn delete_team(id: Uuid) -> Result<Option<()>> {
    let result = sqlx::query(&sql("DELETE FROM teams WHERE id = ?"))
        .bind(id)
        .execute(get_pool())
        .await?;

    Ok(if result.rows_affected() > 0 {
        Some(())
    } else {
        None
    })
}

// Team member management
pub async fn add_team_member(team_id: Uuid, user_id: Uuid) -> Result<TeamMember> {
    let now = Utc::now();
    let team_member = sqlx::query_as::<_, TeamMember>(&sql(r#"
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
        "#))
    .bind(team_id)
    .bind(user_id)
    .bind(now)
    .fetch_one(get_pool())
    .await?;

    Ok(team_member)
}

pub async fn get_team_members(team_id: Uuid) -> Result<Vec<TeamMember>> {
    let team_members = sqlx::query_as::<_, TeamMember>(&sql(r#"
            SELECT
                id,
                team_id,
                user_id,
                created_at
            FROM
                team_members
            WHERE
                team_id = ?
        "#))
    .bind(team_id)
    .fetch_all(get_pool())
    .await?;

    Ok(team_members)
}

pub async fn remove_team_member(team_id: Uuid, user_id: Uuid) -> Result<Option<()>> {
    let result = sqlx::query(&sql(r#"
            DELETE FROM team_members
            WHERE
                team_id = ?
                AND user_id = ?
            "#))
    .bind(team_id)
    .bind(user_id)
    .execute(get_pool())
    .await?;

    Ok(if result.rows_affected() > 0 {
        Some(())
    } else {
        None
    })
}

pub async fn get_user_teams(user_id: Uuid) -> Result<Vec<Team>> {
    let teams = sqlx::query_as::<_, Team>(&sql(r#"
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
        "#))
    .bind(user_id)
    .fetch_all(get_pool())
    .await?;

    Ok(teams)
}
