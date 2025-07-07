use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::models::{InviteToken, UserRole};

pub struct InviteRepository {
    pool: SqlitePool,
}

impl InviteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_invite_token(
        &self,
        email: &str,
        inviter_id: &str,
        role: UserRole,
        team_id: Option<i64>,
    ) -> Result<InviteToken, sqlx::Error> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now().naive_utc() + Duration::days(7); // 7 days to accept
        let created_at = Utc::now().naive_utc();

        let invite_token = InviteToken {
            id: Uuid::new_v4().to_string(),
            email: email.to_string(),
            token: token.clone(),
            inviter_id: inviter_id.to_string(),
            role: role.clone(),
            team_id,
            expires_at,
            used_at: None,
            created_at,
        };

        let role_str = role.to_string();
        sqlx::query!(
            r#"
            INSERT INTO invite_tokens (id, email, token, inviter_id, role, team_id, expires_at, used_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            invite_token.id,
            invite_token.email,
            invite_token.token,
            invite_token.inviter_id,
            role_str,
            invite_token.team_id,
            invite_token.expires_at,
            invite_token.used_at,
            invite_token.created_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(invite_token)
    }

    pub async fn get_invite_token(&self, token: &str) -> Result<Option<InviteToken>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, email, token, inviter_id, role, team_id, expires_at, used_at, created_at FROM invite_tokens WHERE token = ? AND used_at IS NULL",
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let role = row.role.parse::<UserRole>().map_err(|e| {
                    sqlx::Error::Decode(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid role: {}", e),
                    )))
                })?;

                Ok(Some(InviteToken {
                    id: row.id.unwrap(),
                    email: row.email,
                    token: row.token,
                    inviter_id: row.inviter_id,
                    role,
                    team_id: row.team_id,
                    expires_at: row.expires_at,
                    used_at: row.used_at,
                    created_at: row.created_at.unwrap_or_else(|| Utc::now().naive_utc()),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn mark_invite_token_as_used(&self, token: &str) -> Result<(), sqlx::Error> {
        let used_at = Utc::now().naive_utc();
        
        sqlx::query!(
            "UPDATE invite_tokens SET used_at = ? WHERE token = ?",
            used_at,
            token
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_invites_by_inviter(&self, inviter_id: &str) -> Result<Vec<InviteToken>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, email, token, inviter_id, role, team_id, expires_at, used_at, created_at FROM invite_tokens WHERE inviter_id = ? ORDER BY created_at DESC",
            inviter_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut invites = Vec::new();
        for row in rows {
            let role = row.role.parse::<UserRole>().map_err(|e| {
                sqlx::Error::Decode(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid role: {}", e),
                )))
            })?;

            invites.push(InviteToken {
                id: row.id.unwrap(),
                email: row.email,
                token: row.token,
                inviter_id: row.inviter_id,
                role,
                team_id: row.team_id,
                expires_at: row.expires_at,
                used_at: row.used_at,
                created_at: row.created_at.unwrap_or_else(|| Utc::now().naive_utc()),
            });
        }

        Ok(invites)
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<u64, sqlx::Error> {
        let now = Utc::now().naive_utc();
        let result = sqlx::query!(
            "DELETE FROM invite_tokens WHERE expires_at < ? AND used_at IS NULL",
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn revoke_invite_token(&self, token: &str, inviter_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM invite_tokens WHERE token = ? AND inviter_id = ? AND used_at IS NULL",
            token,
            inviter_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
