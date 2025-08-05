use anyhow::Result;

use chrono::Utc;
use uuid::Uuid;

use crate::database::{
    models::{ShiftClaim, ShiftClaimInput},
    pool,
    utils::sql,
};

/// Create a new shift claim
pub async fn create_claim(input: &ShiftClaimInput) -> Result<ShiftClaim> {
    let now = Utc::now();
    let status = "pending";

    let claim = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
            INSERT INTO
                shift_claims (
                    shift_id,
                    user_id,
                    status,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?)
            RETURNING
                id,
                shift_id,
                user_id,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
        "#))
    .bind(input.shift_id)
    .bind(input.user_id)
    .bind(status)
    .bind(now)
    .bind(now)
    .fetch_one(pool())
    .await?;

    Ok(claim)
}

/// Get a specific shift claim by ID
pub async fn get_claim_by_id(id: Uuid) -> Result<Option<ShiftClaim>> {
    let claim = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
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
                id = ?
        "#))
    .bind(id)
    .fetch_optional(pool())
    .await?;

    Ok(claim)
}

/// Get all claims for a specific shift
pub async fn get_claims_by_shift(shift_id: Uuid) -> Result<Vec<ShiftClaim>> {
    let claims = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
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
                shift_id = ?
            ORDER BY
                created_at DESC
        "#))
    .bind(shift_id)
    .fetch_all(pool())
    .await?;

    Ok(claims)
}

/// Get all claims by a specific user
pub async fn get_claims_by_user(user_id: Uuid) -> Result<Vec<ShiftClaim>> {
    let claims = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
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
                user_id = ?
            ORDER BY
                created_at DESC
        "#))
    .bind(user_id)
    .fetch_all(pool())
    .await?;

    Ok(claims)
}

/// Get all claims
pub async fn get_all_claims_by_company(company_id: Uuid) -> Result<Vec<ShiftClaim>> {
    let claims = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
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
                company_id = ?
            ORDER BY
                created_at DESC
        "#))
    .bind(company_id)
    .fetch_all(pool())
    .await?;

    Ok(claims)
}

/// Get pending claims for approval (managers/admins)
pub async fn get_pending_claims_by_company(company_id: Uuid) -> Result<Vec<ShiftClaim>> {
    let claims = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
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
                AND company_id = ?
            ORDER BY
                created_at ASC
        "#))
    .bind(company_id)
    .fetch_all(pool())
    .await?;

    Ok(claims)
}

/// Approve a shift claim
pub async fn approve_claim(
    claim_id: Uuid,
    approved_by: Uuid,
    approval_notes: Option<String>,
) -> Result<Option<ShiftClaim>> {
    let now = Utc::now();

    let claim = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
            UPDATE
                shift_claims
            SET
                status = 'approved',
                approved_by = ?,
                approval_notes = ?,
                updated_at = ?
            WHERE
                id = ?
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
        "#))
    .bind(approved_by)
    .bind(approval_notes)
    .bind(now)
    .bind(claim_id)
    .fetch_optional(pool())
    .await?;

    Ok(claim)
}

/// Reject a shift claim
pub async fn reject_claim(
    claim_id: Uuid,
    approved_by: Uuid,
    approval_notes: Option<String>,
) -> Result<Option<ShiftClaim>> {
    let now = Utc::now();

    let claim = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
            UPDATE
                shift_claims
            SET
                status = 'rejected',
                approved_by = ?,
                approval_notes = ?,
                updated_at = ?
            WHERE
                id = ?
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
        "#))
    .bind(approved_by)
    .bind(approval_notes)
    .bind(now)
    .bind(claim_id)
    .fetch_optional(pool())
    .await?;

    Ok(claim)
}

/// Cancel a shift claim (can only be done by the claim owner)
pub async fn cancel_claim(claim_id: Uuid, user_id: Uuid) -> Result<Option<ShiftClaim>> {
    let now = Utc::now();

    let claim = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
            UPDATE
                shift_claims
            SET
                status = 'cancelled',
                updated_at = ?
            WHERE
                id = ?
                AND user_id = ?
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
        "#))
    .bind(now)
    .bind(claim_id)
    .bind(user_id)
    .fetch_optional(pool())
    .await?;

    Ok(claim)
}

/// Cancel all pending claims for a shift (when shift is assigned manually)
pub async fn cancel_pending_claims_for_shift(shift_id: Uuid) -> Result<u64> {
    let now = Utc::now();

    let result = sqlx::query(&sql(r#"
            UPDATE
                shift_claims
            SET
                status = 'cancelled',
                updated_at = ?
            WHERE
                shift_id = ?
                AND status = 'pending'
        "#))
    .bind(now)
    .bind(shift_id)
    .execute(pool())
    .await?;

    Ok(result.rows_affected())
}

/// Check if a user has an active (non-cancelled) claim for a shift
pub async fn has_active_claim(shift_id: Uuid, user_id: Uuid) -> Result<Option<()>> {
    let count: i64 = sqlx::query_scalar(&sql(r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = ?
                AND user_id = ?
                AND status != 'cancelled'
        "#))
    .bind(shift_id)
    .bind(user_id)
    .fetch_one(pool())
    .await?;

    if count > 0 {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

/// Check if a user has a pending claim for a shift
pub async fn has_pending_claim(shift_id: Uuid, user_id: Uuid) -> Result<Option<()>> {
    let count: i64 = sqlx::query_scalar(&sql(r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = ?
                AND user_id = ?
                AND status = 'pending'
        "#))
    .bind(shift_id)
    .bind(user_id)
    .fetch_one(pool())
    .await?;

    if count > 0 {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

/// Check if a user has already claimed a specific shift (pending or approved)
pub async fn has_user_claimed_shift(shift_id: Uuid, user_id: Uuid) -> Result<Option<()>> {
    let count: i64 = sqlx::query_scalar(&sql(r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = ?
                AND user_id = ?
                AND (
                    status = 'pending'
                    OR status = 'approved'
                )
        "#))
    .bind(shift_id)
    .bind(user_id)
    .fetch_one(pool())
    .await?;

    if count > 0 {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

/// Get the approved claim for a shift (if any)
pub async fn get_approved_claim_for_shift(shift_id: Uuid) -> Result<Option<ShiftClaim>> {
    let claim = sqlx::query_as::<_, ShiftClaim>(&sql(r#"
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
                shift_id = ?
                AND status = 'approved'
        "#))
    .bind(shift_id)
    .fetch_optional(pool())
    .await?;

    Ok(claim)
}

/// Check if a shift already has an approved claim
pub async fn has_approved_claim(shift_id: Uuid) -> Result<Option<()>> {
    let count: i64 = sqlx::query_scalar(&sql(r#"
            SELECT
                COUNT(*)
            FROM
                shift_claims
            WHERE
                shift_id = ?
                AND status = 'approved'
        "#))
    .bind(shift_id)
    .fetch_one(pool())
    .await?;

    if count > 0 {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

/// Check if user is a team member for the shift's team
pub async fn is_user_team_member(shift_id: Uuid, user_id: Uuid) -> Result<Option<()>> {
    let count: i64 = sqlx::query_scalar(&sql(r#"
            SELECT
                COUNT(*)
            FROM
                shifts s
                INNER JOIN team_members tm ON s.team_id = tm.team_id
            WHERE
                s.id = ?
                AND tm.user_id = ?
        "#))
    .bind(shift_id)
    .bind(user_id)
    .fetch_one(pool())
    .await?;

    if count > 0 {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}
