use crate::database::models::User;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, name, role, pto_balance_hours, sick_balance_hours, personal_balance_hours, pto_accrual_rate, hire_date, last_accrual_date, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(&user.role.to_string())
        .bind(&user.pto_balance_hours)
        .bind(&user.sick_balance_hours)
        .bind(&user.personal_balance_hours)
        .bind(&user.pto_accrual_rate)
        .bind(&user.hire_date)
        .bind(&user.last_accrual_date)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, role, pto_balance_hours, sick_balance_hours, personal_balance_hours, pto_accrual_rate, hire_date, last_accrual_date, created_at, updated_at
            FROM users
            WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, role, pto_balance_hours, sick_balance_hours, personal_balance_hours, pto_accrual_rate, hire_date, last_accrual_date, created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&self.pool)
            .await?;

        Ok(count > 0)
    }

    pub async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<()> {
        use chrono::Utc;

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
