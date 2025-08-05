use anyhow::Result;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::database::{
    models::{CompanyRole, InviteToken},
    pool,
    utils::sql,
};

pub async fn create_invite_token(
    email: &str,
    inviter_id: Uuid, // Now takes UUID directly
    role: CompanyRole,
    company_id: Uuid,      // UUID for company references
    team_id: Option<Uuid>, // Now takes UUID directly
) -> Result<InviteToken> {
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7); // 7 days to accept
    let created_at = Utc::now();

    let invite_token = sqlx::query_as::<_, InviteToken>(&sql(r#"
            INSERT INTO
                invite_tokens (
                    email,
                    token,
                    inviter_id,
                    role,
                    company_id,
                    team_id,
                    expires_at,
                    created_at
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id,
                email,
                token,
                inviter_id,
                role,
                company_id,
                team_id,
                expires_at,
                used_at,
                created_at
        "#))
    .bind(email)
    .bind(token)
    .bind(inviter_id)
    .bind(role)
    .bind(company_id)
    .bind(team_id)
    .bind(expires_at)
    .bind(created_at)
    .fetch_one(pool())
    .await?;

    Ok(invite_token)
}

pub async fn get_invite_token(token: &str) -> Result<Option<InviteToken>> {
    let invite_token = sqlx::query_as::<_, InviteToken>(&sql(r#"
            SELECT
                id,
                email,
                token,
                inviter_id,
                role,
                company_id,
                team_id,
                expires_at,
                used_at,
                created_at
            FROM
                invite_tokens
            WHERE
                token = ?
                AND used_at IS NULL
        "#))
    .bind(token)
    .fetch_optional(pool())
    .await?;

    Ok(invite_token)
}

pub async fn mark_invite_token_as_used(token: &str) -> Result<()> {
    let used_at = Utc::now();

    sqlx::query(&sql("UPDATE invite_tokens SET used_at = ? WHERE token = ?"))
        .bind(used_at)
        .bind(token)
        .execute(pool())
        .await?;

    Ok(())
}

pub async fn get_invites_by_inviter(
    inviter_id: Uuid,
    company_id: Uuid,
) -> Result<Vec<InviteToken>> {
    let invites = sqlx::query_as::<_, InviteToken>(&sql(r#"
            SELECT
                id,
                email,
                token,
                inviter_id,
                role,
                company_id,
                team_id,
                expires_at,
                used_at,
                created_at
            FROM invite_tokens
            WHERE inviter_id = ?
                AND company_id = ?
            ORDER BY created_at DESC
        "#))
    .bind(inviter_id)
    .bind(company_id)
    .fetch_all(pool())
    .await?;

    Ok(invites)
}

pub async fn cleanup_expired_tokens() -> Result<u64> {
    let now = Utc::now();
    let result = sqlx::query(&sql(
        r#"DELETE FROM invite_tokens WHERE expires_at < ? AND used_at IS NULL"#,
    ))
    .bind(now)
    .execute(pool())
    .await?;

    Ok(result.rows_affected())
}

pub async fn revoke_invite_token(token: &str, inviter_id: Uuid) -> Result<bool> {
    let result = sqlx::query(&sql(r#"
            DELETE FROM invite_tokens
            WHERE
                token = ?
                AND inviter_id = ?
                AND used_at IS NULL
        "#))
    .bind(token)
    .bind(inviter_id)
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}
