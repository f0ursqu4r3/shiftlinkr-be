use crate::database::models::{ShiftClaim, ShiftClaimInput};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ShiftClaimRepository {
    pool: PgPool,
}

impl ShiftClaimRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new shift claim
    pub async fn create_claim(&self, input: &ShiftClaimInput) -> Result<ShiftClaim, sqlx::Error> {
        let now = Utc::now();
        let status = "pending";

        let claim = sqlx::query_as::<_, ShiftClaim>(
            r#"
            INSERT INTO
                shift_claims (
                    shift_id,
                    user_id,
                    status,
                    created_at,
                    updated_at
                )
            VALUES
                ($1, $2, $3, $4, $5)
            RETURNING
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(input.shift_id)
        .bind(input.user_id)
        .bind(status)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(claim)
    }

    /// Get a specific shift claim by ID
    pub async fn get_claim_by_id(&self, id: Uuid) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let claim = sqlx::query_as::<_, ShiftClaim>(
            r#"
            SELECT
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_claims
            WHERE
                id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(claim)
    }

    /// Get all claims for a specific shift
    pub async fn get_claims_by_shift(
        &self,
        shift_id: Uuid,
    ) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let claims = sqlx::query_as::<_, ShiftClaim>(
            r#"
            SELECT
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_claims
            WHERE
                shift_id = $1
            ORDER BY
                created_at DESC
            "#,
        )
        .bind(shift_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(claims)
    }

    /// Get all claims by a specific user
    pub async fn get_claims_by_user(&self, user_id: Uuid) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let claims = sqlx::query_as::<_, ShiftClaim>(
            r#"
            SELECT
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_claims
            WHERE
                user_id = $1
            ORDER BY
                created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(claims)
    }

    /// Get all claims
    pub async fn get_all_claims_by_company(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let claims = sqlx::query_as::<_, ShiftClaim>(
            r#"
            SELECT
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_claims
            WHERE
                company_id = $1
            ORDER BY
                created_at DESC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(claims)
    }

    /// Get pending claims for approval (managers/admins)
    pub async fn get_pending_claims_by_company(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<ShiftClaim>, sqlx::Error> {
        let claims = sqlx::query_as::<_, ShiftClaim>(
            r#"
            SELECT
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_claims
            WHERE
                status = 'pending'
                AND company_id = $1
            ORDER BY
                created_at ASC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(claims)
    }

    /// Approve a shift claim
    pub async fn approve_claim(
        &self,
        claim_id: Uuid,
        approved_by: Uuid,
        approval_notes: Option<String>,
    ) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let now = Utc::now();

        let claim = sqlx::query_as::<_, ShiftClaim>(
            r#"
            UPDATE
                shift_claims
            SET
                status = 'approved',
                approved_by = $1,
                approval_notes = $2,
                updated_at = $3
            WHERE
                id = $4
                AND status = 'pending'
            RETURNING
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(approved_by)
        .bind(approval_notes)
        .bind(now)
        .bind(claim_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(claim)
    }

    /// Reject a shift claim
    pub async fn reject_claim(
        &self,
        claim_id: Uuid,
        approved_by: Uuid,
        approval_notes: Option<String>,
    ) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let now = Utc::now();

        let claim = sqlx::query_as::<_, ShiftClaim>(
            r#"
            UPDATE
                shift_claims
            SET
                status = 'rejected',
                approved_by = $1,
                approval_notes = $2,
                updated_at = $3
            WHERE
                id = $4
                AND status = 'pending'
            RETURNING
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(approved_by)
        .bind(approval_notes)
        .bind(now)
        .bind(claim_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(claim)
    }

    /// Cancel a shift claim (can only be done by the claim owner)
    pub async fn cancel_claim(
        &self,
        claim_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let now = Utc::now();

        let claim = sqlx::query_as::<_, ShiftClaim>(
            r#"
            UPDATE
                shift_claims
            SET
                status = 'cancelled',
                updated_at = $1
            WHERE
                id = $2
                AND user_id = $3
                AND status = 'pending'
            RETURNING
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(now)
        .bind(claim_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(claim)
    }

    /// Cancel all pending claims for a shift (when shift is assigned manually)
    pub async fn cancel_pending_claims_for_shift(
        &self,
        shift_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE
                shift_claims
            SET
                status = 'cancelled',
                updated_at = $1
            WHERE
                shift_id = $2
                AND status = 'pending'
            "#,
        )
        .bind(now)
        .bind(shift_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if a user has an active (non-cancelled) claim for a shift
    pub async fn has_active_claim(
        &self,
        shift_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = $1
                AND user_id = $2
                AND status != 'cancelled'
            "#,
        )
        .bind(shift_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if a user has a pending claim for a shift
    pub async fn has_pending_claim(
        &self,
        shift_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = $1
                AND user_id = $2
                AND status = 'pending'
            "#,
        )
        .bind(shift_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if a user has already claimed a specific shift (pending or approved)
    pub async fn has_user_claimed_shift(
        &self,
        shift_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = $1
                AND user_id = $2
                AND (
                    status = 'pending'
                    OR status = 'approved'
                )
            "#,
        )
        .bind(shift_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Get the approved claim for a shift (if any)
    pub async fn get_approved_claim_for_shift(
        &self,
        shift_id: Uuid,
    ) -> Result<Option<ShiftClaim>, sqlx::Error> {
        let claim = sqlx::query_as::<_, ShiftClaim>(
            r#"
            SELECT
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_claims
            WHERE
                shift_id = $1
                AND status = 'approved'
            "#,
        )
        .bind(shift_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(claim)
    }

    /// Check if a shift already has an approved claim
    pub async fn has_approved_claim(&self, shift_id: Uuid) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = $1
                AND status = 'approved'
            "#,
        )
        .bind(shift_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if user is a team member for the shift's team
    pub async fn is_user_team_member(
        &self,
        shift_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT
                COUNT(*)
            FROM
                shifts s
                INNER JOIN team_members tm ON s.team_id = tm.team_id
            WHERE
                s.id = $1
                AND tm.user_id = $2
            "#,
        )
        .bind(shift_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }
}
