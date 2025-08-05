use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::{
    get_pool,
    models::{Location, LocationInput},
    utils::sql,
};

// Location management methods
pub async fn create_location(location: LocationInput) -> Result<Location> {
    let now = Utc::now();
    let location = sqlx::query_as::<_, Location>(&sql(r#"
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
    "#))
    .bind(location.name)
    .bind(location.address)
    .bind(location.phone)
    .bind(location.email)
    .bind(location.company_id)
    .bind(now)
    .bind(now)
    .fetch_one(get_pool())
    .await?;

    Ok(location)
}

pub async fn find_by_id(id: Uuid) -> Result<Option<Location>> {
    let location = sqlx::query_as::<_, Location>(&sql(r#"
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
            id = ?
    "#))
    .bind(id)
    .fetch_optional(get_pool())
    .await?;

    Ok(location)
}

pub async fn find_by_team_id(team_id: Uuid) -> Result<Option<Location>> {
    let location = sqlx::query_as::<_, Location>(&sql(r#"
        SELECT
            l.id,
            l.name,
            l.address,
            l.phone,
            l.email,
            l.company_id,
            l.created_at,
            l.updated_at
        FROM
            locations l
            INNER JOIN teams t ON l.id = t.location_id
        WHERE
            t.id = ?
    "#))
    .bind(team_id)
    .fetch_optional(get_pool())
    .await?;

    Ok(location)
}

pub async fn get_locations_by_company(company_id: Uuid) -> Result<Vec<Location>> {
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
            company_id = ?
        ORDER BY
            name
    "#))
    .bind(company_id)
    .fetch_all(get_pool())
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
    .fetch_all(get_pool())
    .await?;

    Ok(locations)
}

pub async fn update_location(id: Uuid, input: LocationInput) -> Result<Option<Location>> {
    let now = Utc::now();
    let location = sqlx::query_as::<_, Location>(&sql(r#"
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
    "#))
    .bind(input.name)
    .bind(input.address)
    .bind(input.phone)
    .bind(input.email)
    .bind(input.company_id)
    .bind(now)
    .bind(id)
    .fetch_optional(get_pool())
    .await?;

    Ok(location)
}

pub async fn delete_location(id: Uuid) -> Result<Option<()>> {
    let result = sqlx::query(&sql("DELETE FROM locations WHERE id = ?"))
        .bind(id)
        .execute(get_pool())
        .await?;

    Ok(if result.rows_affected() > 0 {
        Some(())
    } else {
        None
    })
}
