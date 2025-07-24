use crate::database::models::User;
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, name, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, created_at, updated_at
            FROM users
            WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn update_user(&self, id: Uuid, name: &str, email: &str) -> Result<()> {
        let updated_at = Utc::now().naive_utc();

        sqlx::query(
            r#"
            UPDATE users
            SET name = ?, email = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(email)
        .bind(updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&self.pool)
            .await?;

        Ok(count > 0)
    }

    pub async fn update_password(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        let updated_at = Utc::now().naive_utc();

        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(password_hash)
        .bind(updated_at)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
