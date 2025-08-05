use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::{get_pool, models::User, utils::sql};

pub async fn create_user(user: &User) -> Result<User> {
    let user = sqlx::query_as::<_, User>(&sql(r#"
        INSERT INTO
            users (
                id,
                email,
                password_hash,
                name,
                created_at,
                updated_at
            )
        VALUES
            (?, ?, ?, ?, ?, ?)
        RETURNING 
            id,
            email,
            password_hash,
            name,
            created_at,
            updated_at
    "#))
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(&user.name)
    .bind(&user.created_at)
    .bind(&user.updated_at)
    .fetch_one(get_pool())
    .await?;

    Ok(user)
}

pub async fn find_by_email(email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(&sql(r#"
        SELECT
            id,
            email,
            password_hash,
            name,
            created_at,
            updated_at
        FROM
            users
        WHERE
            email = ?
    "#))
    .bind(email)
    .fetch_optional(get_pool())
    .await?;

    Ok(user)
}

pub async fn find_by_id(id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(&sql(r#"
        SELECT
            id,
            email,
            password_hash,
            name,
            created_at,
            updated_at
        FROM
            users
        WHERE
            id = ?
    "#))
    .bind(id)
    .fetch_optional(get_pool())
    .await?;

    Ok(user)
}

pub async fn get_all_users() -> Result<Vec<User>> {
    let users = sqlx::query_as::<_, User>(&sql(r#"
        SELECT
            id,
            email,
            password_hash,
            name,
            created_at,
            updated_at
        FROM
            users
        ORDER BY
            created_at DESC
    "#))
    .fetch_all(get_pool())
    .await?;

    Ok(users)
}

pub async fn update_user(id: Uuid, name: &str, email: &str) -> Result<User> {
    let updated_at = Utc::now().naive_utc();

    let user = sqlx::query_as::<_, User>(&sql(r#"
        UPDATE users
        SET
            name = ?,
            email = ?,
            updated_at = ?
        WHERE
            id = ?
        RETURNING
            id,
            email,
            password_hash,
            name,
            updated_at
    "#))
    .bind(name)
    .bind(email)
    .bind(updated_at)
    .bind(id)
    .fetch_one(get_pool())
    .await?;

    Ok(user)
}

pub async fn delete_user(id: Uuid) -> Result<()> {
    sqlx::query(&sql(r#"
        DELETE FROM users
        WHERE
            id = ?
    "#))
    .bind(id)
    .execute(get_pool())
    .await?;

    Ok(())
}

pub async fn email_exists(email: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(&sql(r#"
        SELECT
            COUNT(*)
        FROM
            users
        WHERE
            email = ?
    "#))
    .bind(email)
    .fetch_one(get_pool())
    .await?;

    Ok(count > 0)
}

pub async fn update_password(user_id: Uuid, password_hash: &str) -> Result<()> {
    let updated_at = Utc::now().naive_utc();

    sqlx::query(&sql(r#"
        UPDATE users
        SET
            password_hash = ?,
            updated_at = ?
        WHERE
            id = ?
    "#))
    .bind(password_hash)
    .bind(updated_at)
    .bind(user_id)
    .execute(get_pool())
    .await?;

    Ok(())
}
