use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::PasswordResetToken;

#[derive(Clone)]
pub struct PasswordResetTokenRepository {
    pool: PgPool,
}

impl PasswordResetTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new password reset token
    pub async fn create_token(&self, user_id: &str) -> Result<PasswordResetToken> {
        let token_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now().naive_utc() + Duration::hours(1); // 1 hour expiration
        let created_at = Utc::now().naive_utc();

        let reset_token = PasswordResetToken {
            id: token_id.clone(),
            user_id: user_id.to_string(),
            token: token.clone(),
            expires_at,
            used_at: None,
            created_at,
        };

        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&reset_token.id)
        .bind(&reset_token.user_id)
        .bind(&reset_token.token)
        .bind(&reset_token.expires_at)
        .bind(&reset_token.created_at)
        .execute(&self.pool)
        .await?;

        Ok(reset_token)
    }

    /// Find a valid (unused and not expired) token
    pub async fn find_valid_token(&self, token: &str) -> Result<Option<PasswordResetToken>> {
        let now = Utc::now().naive_utc();

        let result = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT id, user_id, token, expires_at, used_at, created_at
            FROM password_reset_tokens
            WHERE token = $1 AND used_at IS NULL AND expires_at > $2
            "#,
        )
        .bind(token)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Mark a token as used
    pub async fn mark_token_used(&self, token: &str) -> Result<()> {
        let now = Utc::now().naive_utc();

        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = $1
            WHERE token = $2
            "#,
        )
        .bind(now)
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired tokens (optional cleanup method)
    pub async fn cleanup_expired_tokens(&self) -> Result<u64> {
        let now = Utc::now().naive_utc();

        let result = sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE expires_at < $1
            "#,
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Invalidate all tokens for a user (when password is reset)
    pub async fn invalidate_user_tokens(&self, user_id: &str) -> Result<()> {
        let now = Utc::now().naive_utc();

        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = $1
            WHERE user_id = $2 AND used_at IS NULL
            "#,
        )
        .bind(now)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
