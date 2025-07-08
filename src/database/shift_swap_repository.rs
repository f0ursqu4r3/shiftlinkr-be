use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::database::models::{ShiftSwap, ShiftSwapInput, ShiftSwapStatus, ShiftSwapType};

#[derive(Clone)]
pub struct ShiftSwapRepository {
    pool: SqlitePool,
}

impl ShiftSwapRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new shift swap request
    pub async fn create_swap_request(&self, input: ShiftSwapInput) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();
        
        // Determine initial status based on swap type
        let initial_status = match input.swap_type {
            ShiftSwapType::Open => ShiftSwapStatus::Open,
            ShiftSwapType::Targeted => ShiftSwapStatus::Pending,
        };

        let swap = sqlx::query_as!(
            ShiftSwap,
            r#"
            INSERT INTO shift_swaps (
                original_shift_id, requesting_user_id, target_user_id, 
                status, notes, swap_type, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, original_shift_id, requesting_user_id, target_user_id,
                status as "status: ShiftSwapStatus",
                notes,
                swap_type as "swap_type: ShiftSwapType",
                approved_by, approval_notes, created_at, updated_at
            "#,
            input.original_shift_id,
            input.requesting_user_id,
            input.target_user_id,
            initial_status.to_string(),
            input.notes,
            input.swap_type.to_string(),
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(swap)
    }

    /// Get all swap requests with optional filtering
    pub async fn get_swap_requests(
        &self,
        user_id: Option<&str>,
        status: Option<ShiftSwapStatus>,
        swap_type: Option<ShiftSwapType>,
    ) -> Result<Vec<ShiftSwap>> {
        let mut query = "
            SELECT 
                id, original_shift_id, requesting_user_id, target_user_id,
                status, notes, swap_type, approved_by, approval_notes,
                created_at, updated_at
            FROM shift_swaps
            WHERE 1=1
        ".to_string();

        let mut params: Vec<String> = Vec::new();

        if let Some(uid) = user_id {
            query.push_str(" AND (requesting_user_id = ? OR target_user_id = ?)");
            params.push(uid.to_string());
            params.push(uid.to_string());
        }

        if let Some(s) = status {
            query.push_str(" AND status = ?");
            params.push(s.to_string());
        }

        if let Some(st) = swap_type {
            query.push_str(" AND swap_type = ?");
            params.push(st.to_string());
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut sql_query = sqlx::query_as::<_, ShiftSwap>(&query);
        
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let swaps = sql_query.fetch_all(&self.pool).await?;
        Ok(swaps)
    }

    /// Get open swap requests (available for anyone to respond to)
    pub async fn get_open_swap_requests(&self, exclude_user_id: Option<&str>) -> Result<Vec<ShiftSwap>> {
        let mut query = "
            SELECT 
                id, original_shift_id, requesting_user_id, target_user_id,
                status, notes, swap_type, approved_by, approval_notes,
                created_at, updated_at
            FROM shift_swaps
            WHERE status = 'open' AND swap_type = 'open'
        ".to_string();

        let swap_requests = if let Some(uid) = exclude_user_id {
            query.push_str(" AND requesting_user_id != ?");
            query.push_str(" ORDER BY created_at DESC");
            sqlx::query_as::<_, ShiftSwap>(&query)
                .bind(uid)
                .fetch_all(&self.pool)
                .await?
        } else {
            query.push_str(" ORDER BY created_at DESC");
            sqlx::query_as::<_, ShiftSwap>(&query)
                .fetch_all(&self.pool)
                .await?
        };

        Ok(swap_requests)
    }

    /// Get a specific swap request by ID
    pub async fn get_swap_by_id(&self, id: i64) -> Result<Option<ShiftSwap>> {
        let swap = sqlx::query_as!(
            ShiftSwap,
            r#"
            SELECT 
                id, original_shift_id, requesting_user_id, target_user_id,
                status as "status: ShiftSwapStatus",
                notes,
                swap_type as "swap_type: ShiftSwapType",
                approved_by, approval_notes, created_at, updated_at
            FROM shift_swaps 
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(swap)
    }

    /// Respond to a swap request (accept/decline)
    pub async fn respond_to_swap(&self, id: i64, responding_user_id: &str, accept: bool, notes: Option<String>) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();
        let new_status = if accept { ShiftSwapStatus::Pending } else { ShiftSwapStatus::Open };

        let swap = if accept {
            // Accept the swap - set target user and move to pending for manager approval
            sqlx::query_as!(
                ShiftSwap,
                r#"
                UPDATE shift_swaps 
                SET 
                    target_user_id = ?, status = ?, notes = ?, updated_at = ?
                WHERE id = ?
                RETURNING 
                    id, original_shift_id, requesting_user_id, target_user_id,
                    status as "status: ShiftSwapStatus",
                    notes,
                    swap_type as "swap_type: ShiftSwapType",
                    approved_by, approval_notes, created_at, updated_at
                "#,
                responding_user_id,
                new_status.to_string(),
                notes,
                now,
                id
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            // Decline - just update timestamp and possibly add notes
            sqlx::query_as!(
                ShiftSwap,
                r#"
                UPDATE shift_swaps 
                SET updated_at = ?
                WHERE id = ?
                RETURNING 
                    id, original_shift_id, requesting_user_id, target_user_id,
                    status as "status: ShiftSwapStatus",
                    notes,
                    swap_type as "swap_type: ShiftSwapType",
                    approved_by, approval_notes, created_at, updated_at
                "#,
                now,
                id
            )
            .fetch_one(&self.pool)
            .await?
        };

        Ok(swap)
    }

    /// Approve a swap request (manager action)
    pub async fn approve_swap(&self, id: i64, approved_by: &str, notes: Option<String>) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();

        let swap = sqlx::query_as!(
            ShiftSwap,
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, original_shift_id, requesting_user_id, target_user_id,
                status as "status: ShiftSwapStatus",
                notes,
                swap_type as "swap_type: ShiftSwapType",
                approved_by, approval_notes, created_at, updated_at
            "#,
            ShiftSwapStatus::Approved.to_string(),
            approved_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(swap)
    }

    /// Deny a swap request (manager action)
    pub async fn deny_swap(&self, id: i64, denied_by: &str, notes: String) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();

        let swap = sqlx::query_as!(
            ShiftSwap,
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, original_shift_id, requesting_user_id, target_user_id,
                status as "status: ShiftSwapStatus",
                notes,
                swap_type as "swap_type: ShiftSwapType",
                approved_by, approval_notes, created_at, updated_at
            "#,
            ShiftSwapStatus::Denied.to_string(),
            denied_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(swap)
    }

    /// Cancel a swap request
    pub async fn cancel_swap(&self, id: i64) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();

        let swap = sqlx::query_as!(
            ShiftSwap,
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, original_shift_id, requesting_user_id, target_user_id,
                status as "status: ShiftSwapStatus",
                notes,
                swap_type as "swap_type: ShiftSwapType",
                approved_by, approval_notes, created_at, updated_at
            "#,
            ShiftSwapStatus::Cancelled.to_string(),
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(swap)
    }

    /// Complete a swap (after successful shift reassignment)
    pub async fn complete_swap(&self, id: i64) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();

        let swap = sqlx::query_as!(
            ShiftSwap,
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, original_shift_id, requesting_user_id, target_user_id,
                status as "status: ShiftSwapStatus",
                notes,
                swap_type as "swap_type: ShiftSwapType",
                approved_by, approval_notes, created_at, updated_at
            "#,
            ShiftSwapStatus::Completed.to_string(),
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(swap)
    }

    /// Check if user has pending swap requests for a shift
    pub async fn has_pending_swap_for_shift(&self, shift_id: i64, user_id: &str) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "
            SELECT COUNT(*) as count
            FROM shift_swaps 
            WHERE original_shift_id = ? 
                AND requesting_user_id = ?
                AND status IN ('open', 'pending')
            ",
            shift_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Get swap statistics
    pub async fn get_swap_stats(&self, user_id: Option<&str>) -> Result<(i64, i64, i64, i64, i64, i64)> {
        let mut query = "
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN status = 'open' THEN 1 ELSE 0 END) as open,
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END) as approved,
                SUM(CASE WHEN status = 'denied' THEN 1 ELSE 0 END) as denied,
                SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as completed
            FROM shift_swaps
        ".to_string();

        let stats = if let Some(uid) = user_id {
            query.push_str(" WHERE requesting_user_id = ? OR target_user_id = ?");
            sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64)>(&query)
                .bind(uid)
                .bind(uid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64)>(&query)
                .fetch_one(&self.pool)
                .await?
        };

        Ok(stats)
    }

    /// Get swaps involving a specific user
    pub async fn get_user_swaps(&self, user_id: &str, include_completed: bool) -> Result<Vec<ShiftSwap>> {
        let mut query = "
            SELECT 
                id, original_shift_id, requesting_user_id, target_user_id,
                status, notes, swap_type, approved_by, approval_notes,
                created_at, updated_at
            FROM shift_swaps
            WHERE (requesting_user_id = ? OR target_user_id = ?)
        ".to_string();

        if !include_completed {
            query.push_str(" AND status NOT IN ('completed', 'cancelled')");
        }

        query.push_str(" ORDER BY created_at DESC");

        let swaps = sqlx::query_as::<_, ShiftSwap>(&query)
            .bind(user_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(swaps)
    }
}
