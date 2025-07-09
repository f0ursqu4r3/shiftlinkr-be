use crate::database::models::{ShiftClaim, ShiftClaimInput, ShiftClaimStatus};
use chrono::Utc;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ShiftClaimRepository {
    pool: SqlitePool,
}

impl ShiftClaimRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new shift claim
    pub async fn create_claim(&self, input: &ShiftClaimInput) -> Result<ShiftClaim, sqlx::Error> {
        let now = Utc::now().naive_utc();
        let status = "pending";

        let row = sqlx::query!(
            r#"
            INSERT INTO shift_claims (shift_id, user_id, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            "#,
            input.shift_id,
            input.user_id,
            status,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        let claim = ShiftClaim {
            id: row.id,
            shift_id: row.shift_id,
            user_id: row.user_id,
            status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        Ok(claim)
    }

    /// Get a specific shift claim by ID
    pub async fn get_claim_by_id(&self, id: i64) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            FROM shift_claims
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all claims for a specific shift
    pub async fn get_claims_by_shift(&self, shift_id: i64) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            FROM shift_claims
            WHERE shift_id = ?
            ORDER BY created_at DESC
            "#,
            shift_id
        )
        .fetch_all(&self.pool)
        .await?;

        let claims = rows
            .into_iter()
            .map(|row| ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(claims)
    }

    /// Get all claims by a specific user
    pub async fn get_claims_by_user(&self, user_id: &str) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            FROM shift_claims
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let claims = rows
            .into_iter()
            .map(|row| ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(claims)
    }

    /// Get all claims
    pub async fn get_all_claims(&self) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            FROM shift_claims
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let claims = rows
            .into_iter()
            .map(|row| ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(claims)
    }

    /// Get pending claims for approval (managers/admins)
    pub async fn get_pending_claims(&self) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            FROM shift_claims
            WHERE status = 'pending'
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let claims = rows
            .into_iter()
            .map(|row| ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(claims)
    }

    /// Approve a shift claim
    pub async fn approve_claim(
        &self,
        claim_id: i64,
        approved_by: &str,
        approval_notes: Option<String>,
    ) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let now = Utc::now().naive_utc();

        let row = sqlx::query!(
            r#"
            UPDATE shift_claims
            SET status = 'approved', approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ? AND status = 'pending'
            RETURNING id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            "#,
            approved_by,
            approval_notes,
            now,
            claim_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Reject a shift claim
    pub async fn reject_claim(
        &self,
        claim_id: i64,
        approved_by: &str,
        approval_notes: Option<String>,
    ) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let now = Utc::now().naive_utc();

        let row = sqlx::query!(
            r#"
            UPDATE shift_claims
            SET status = 'rejected', approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ? AND status = 'pending'
            RETURNING id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            "#,
            approved_by,
            approval_notes,
            now,
            claim_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Cancel a shift claim (can only be done by the claim owner)
    pub async fn cancel_claim(
        &self,
        claim_id: i64,
        user_id: &str,
    ) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let now = Utc::now().naive_utc();

        let row = sqlx::query!(
            r#"
            UPDATE shift_claims
            SET status = 'cancelled', updated_at = ?
            WHERE id = ? AND user_id = ? AND status = 'pending'
            RETURNING id, shift_id, user_id, status, approved_by, approval_notes, created_at, updated_at
            "#,
            now,
            claim_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ShiftClaim {
                id: row.id,
                shift_id: row.shift_id,
                user_id: row.user_id,
                status: row.status.parse().unwrap_or(ShiftClaimStatus::Pending),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Cancel all pending claims for a shift (when shift is assigned manually)
    pub async fn cancel_pending_claims_for_shift(&self, shift_id: i64) -> Result<u64, sqlx::Error> {
        let now = Utc::now().naive_utc();

        let result = sqlx::query!(
            r#"
            UPDATE shift_claims
            SET status = 'cancelled', updated_at = ?
            WHERE shift_id = ? AND status = 'pending'
            "#,
            now,
            shift_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if a user has an active (non-cancelled) claim for a shift
    pub async fn has_active_claim(&self, shift_id: i64, user_id: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM shift_claims
            WHERE shift_id = ? AND user_id = ? AND status != 'cancelled'
            "#,
            shift_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if a user has a pending claim for a shift
    pub async fn has_pending_claim(&self, shift_id: i64, user_id: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM shift_claims
            WHERE shift_id = ? AND user_id = ? AND status = 'pending'
            "#,
            shift_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if a user has already claimed a specific shift (pending or approved)
    pub async fn has_user_claimed_shift(&self, shift_id: i64, user_id: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM shift_claims
            WHERE shift_id = ? AND user_id = ? AND (status = 'pending' OR status = 'approved')
            "#,
            shift_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if a shift already has an approved claim
    pub async fn has_approved_claim(&self, shift_id: i64) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM shift_claims
            WHERE shift_id = ? AND status = 'approved'
            "#,
            shift_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if user is a team member for the shift's team
    pub async fn is_user_team_member(&self, shift_id: i64, user_id: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM shifts s
            INNER JOIN team_members tm ON s.team_id = tm.team_id
            WHERE s.id = ? AND tm.user_id = ?
            "#,
            shift_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Get shift info to validate claim eligibility
    pub async fn get_shift_claim_info(&self, shift_id: i64) -> Result<Option<ShiftClaimInfo>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT s.id, s.team_id, s.assigned_user_id, s.start_time, s.status as shift_status
            FROM shifts s
            WHERE s.id = ?
            "#,
            shift_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ShiftClaimInfo {
                id: row.id,
                team_id: row.team_id,
                assigned_user_id: row.assigned_user_id,
                start_time: row.start_time,
                shift_status: row.shift_status,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShiftClaimInfo {
    pub id: i64,
    pub team_id: Option<i64>,
    pub assigned_user_id: Option<String>,
    pub start_time: chrono::NaiveDateTime,
    pub shift_status: String,
}
