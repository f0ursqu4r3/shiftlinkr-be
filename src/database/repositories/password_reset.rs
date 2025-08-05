use anyhow::Result;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::database::{get_pool, models::PasswordResetToken, utils::sql};

/// Generate a cryptographically secure random token
fn generate_secure_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";
    const TOKEN_LEN: usize = 64;
    let mut rng = rand::rng();

    (0..TOKEN_LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Create a new password reset token
pub async fn create_token(user_id: Uuid) -> Result<PasswordResetToken> {
    let token_id = Uuid::new_v4();
    let token = generate_secure_token();
    let expires_at = Utc::now() + Duration::hours(1); // 1 hour expiration
    let created_at = Utc::now();

    let reset_token = sqlx::query_as::<_, PasswordResetToken>(&sql(r#"
        INSERT INTO
            password_reset_tokens (id, user_id, token, expires_at, created_at)
        VALUES
            (?, ?, ?, ?, ?)
        RETURNING
            id,
            user_id,
            token,
            expires_at,
            used_at,
            created_at
    "#))
    .bind(&token_id)
    .bind(&user_id)
    .bind(&token.clone())
    .bind(&expires_at)
    .bind(&created_at)
    .fetch_one(get_pool())
    .await?;

    Ok(reset_token)
}

/// Find a valid (unused and not expired) token
pub async fn find_valid_token(token: &str) -> Result<Option<PasswordResetToken>> {
    let now = Utc::now();

    let result = sqlx::query_as::<_, PasswordResetToken>(&sql(r#"
        SELECT
            id,
            user_id,
            token,
            expires_at,
            used_at,
            created_at
        FROM
            password_reset_tokens
        WHERE
            token = ?
            AND used_at IS NULL
            AND expires_at > ?
    "#))
    .bind(token)
    .bind(now)
    .fetch_optional(get_pool())
    .await?;

    Ok(result)
}

/// Mark a token as used
pub async fn mark_token_used(token: &str) -> Result<PasswordResetToken> {
    let now = Utc::now();

    let result = sqlx::query_as::<_, PasswordResetToken>(&sql(r#"
        UPDATE password_reset_tokens
        SET
            used_at = ?
        WHERE
            token = ?
            AND used_at IS NULL
        RETURNING
            id,
            user_id,
            token,
            expires_at,
            used_at,
            created_at
    "#))
    .bind(now)
    .bind(token)
    .fetch_one(get_pool())
    .await?;

    Ok(result)
}

/// Clean up expired tokens (optional cleanup method)
pub async fn cleanup_expired_tokens() -> Result<u64> {
    let now = Utc::now();

    let result = sqlx::query(&sql(r#"
        DELETE FROM password_reset_tokens
        WHERE
            expires_at < ?
    "#))
    .bind(now)
    .execute(get_pool())
    .await?;

    Ok(result.rows_affected())
}

/// Invalidate all tokens for a user (when password is reset)
pub async fn invalidate_user_tokens(user_id: Uuid) -> Result<()> {
    let now = Utc::now();

    sqlx::query(&sql(r#"
        UPDATE password_reset_tokens
        SET used_at = ?
        WHERE user_id = ?
        AND used_at IS NULL
    "#))
    .bind(now)
    .bind(user_id)
    .execute(get_pool())
    .await?;

    Ok(())
}
