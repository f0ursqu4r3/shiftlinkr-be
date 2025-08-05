use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::{
    models::{Location, LocationInput},
    pool,
    utils::sql,
};

// Location management methods
pub async fn create_location(location: LocationInput) -> Result<Location> {
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
                ($1, $2, $3, $4, $5, $6, $7)
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
    .bind(location.name)
    .bind(location.address)
    .bind(location.phone)
    .bind(location.email)
    .bind(location.company_id)
    .bind(now)
    .bind(now)
    .fetch_one(pool())
    .await?;

    Ok(location)
}

pub async fn find_by_id(id: Uuid) -> Result<Option<Location>> {
    let location = sqlx::query_as::<_, Location>(
            "SELECT id, name, address, phone, email, company_id, created_at, updated_at FROM locations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool())
        .await?;

    Ok(location)
}

pub async fn find_by_team_id(team_id: Uuid) -> Result<Option<Location>> {
    let location = sqlx::query_as::<_, Location>(
            "SELECT l.id, l.name, l.address, l.phone, l.email, l.company_id, l.created_at, l.updated_at FROM locations l INNER JOIN teams t ON l.id = t.location_id WHERE t.id = $1"
        )
        .bind(team_id)
        .fetch_optional(pool())
        .await?;

    Ok(location)
}

pub async fn get_locations_by_company(company_id: Uuid) -> Result<Vec<Location>> {
    let locations = sqlx::query_as::<_, Location>(
            "SELECT id, name, address, phone, email, company_id, created_at, updated_at FROM locations WHERE company_id = $1 ORDER BY name"
        )
        .bind(company_id)
        .fetch_all(pool())
        .await?;

    Ok(locations)
}

pub async fn get_locations_by_company_ids(company_ids: Vec<Uuid>) -> Result<Vec<Location>> {
    if company_ids.is_empty() {
        return Ok(Vec::new());
    }

    let locations = sqlx::query_as::<_, Location>(&sql(r#"
            SELECT
                id,
                name,
                address,
                phone,
                email,
                company_id,
                created_at,
                updated_at
            FROM
                locations
            WHERE
                company_id = ANY (?)
            ORDER BY
                name
        "#))
    .bind(&company_ids)
    .fetch_all(pool())
    .await?;

    Ok(locations)
}

pub async fn update_location(id: Uuid, input: LocationInput) -> Result<Option<Location>> {
    let now = Utc::now();
    let location = sqlx::query_as::<_, Location>(
        r#"
            UPDATE
                locations
            SET
                name = $1,
                address = $2,
                phone = $3,
                email = $4,
                company_id = $5,
                updated_at = $6
            WHERE
                id = $7
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
    .fetch_optional(pool())
    .await?;

    Ok(location)
}

pub async fn delete_location(id: Uuid) -> Result<Option<()>> {
    let result = sqlx::query("DELETE FROM locations WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await?;

    Ok(if result.rows_affected() > 0 {
        Some(())
    } else {
        None
    })
}
