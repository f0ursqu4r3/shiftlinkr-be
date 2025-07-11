use chrono::Utc;
use sqlx::SqlitePool;

use crate::database::{
    models::{ShiftSwap, ShiftSwapInput, ShiftSwapStatus, ShiftSwapType},
    Result,
};

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

        let swap_type_str = input.swap_type.to_string();
        let status_str = initial_status.to_string();

        let row = sqlx::query!(
            r#"
            INSERT INTO shift_swaps (
                requesting_user_id, original_shift_id, target_user_id, target_shift_id, 
                notes, swap_type, status, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, requesting_user_id, original_shift_id, target_user_id, target_shift_id,
                notes, swap_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            input.requesting_user_id,
            input.original_shift_id,
            input.target_user_id,
            input.target_shift_id,
            input.notes,
            swap_type_str,
            status_str,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ShiftSwap {
            id: row.id,
            requesting_user_id: row.requesting_user_id,
            original_shift_id: row.original_shift_id,
            target_user_id: row.target_user_id,
            target_shift_id: row.target_shift_id,
            notes: row.notes,
            swap_type: row.swap_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Get all swap requests with optional filtering
    pub async fn get_swap_requests(
        &self,
        user_id: Option<&str>,
        status: Option<ShiftSwapStatus>,
        swap_type: Option<ShiftSwapType>,
    ) -> Result<Vec<ShiftSwap>> {
        // For simplicity, just get all and filter in memory for now
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, requesting_user_id, original_shift_id, target_user_id, target_shift_id,
                notes, swap_type, status, approved_by, approval_notes,
                created_at, updated_at
            FROM shift_swaps
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut swaps = Vec::new();
        for row in rows {
            let swap = ShiftSwap {
                id: row.id,
                requesting_user_id: row.requesting_user_id,
                original_shift_id: row.original_shift_id,
                target_user_id: row.target_user_id,
                target_shift_id: row.target_shift_id,
                notes: row.notes,
                swap_type: row.swap_type.parse().unwrap(),
                status: row.status.parse().unwrap(),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at.unwrap(),
                updated_at: row.updated_at.unwrap(),
            };

            // Apply filters
            if let Some(uid) = user_id {
                if swap.requesting_user_id != uid
                    && swap.target_user_id.as_ref() != Some(&uid.to_string())
                {
                    continue;
                }
            }

            if let Some(ref s) = status {
                if &swap.status != s {
                    continue;
                }
            }

            if let Some(ref st) = swap_type {
                if &swap.swap_type != st {
                    continue;
                }
            }

            swaps.push(swap);
        }

        Ok(swaps)
    }

    /// Get a swap request by ID
    pub async fn get_swap_by_id(&self, id: i64) -> Result<Option<ShiftSwap>> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, requesting_user_id, original_shift_id, target_user_id, target_shift_id,
                notes, swap_type, status, approved_by, approval_notes, created_at, updated_at
            FROM shift_swaps 
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(ShiftSwap {
                id: row.id,
                requesting_user_id: row.requesting_user_id,
                original_shift_id: row.original_shift_id,
                target_user_id: row.target_user_id,
                target_shift_id: row.target_shift_id,
                notes: row.notes,
                swap_type: row.swap_type.parse().unwrap(),
                status: row.status.parse().unwrap(),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at.unwrap(),
                updated_at: row.updated_at.unwrap(),
            })),
            None => Ok(None),
        }
    }

    /// Respond to a swap request (accept/decline)
    pub async fn respond_to_swap(
        &self,
        id: i64,
        responding_user_id: &str,
        accept: bool,
        notes: Option<String>,
    ) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();
        let new_status = if accept {
            ShiftSwapStatus::Pending
        } else {
            ShiftSwapStatus::Open
        };
        let status_str = new_status.to_string();

        // Update the swap
        if accept {
            sqlx::query!(
                r#"
                UPDATE shift_swaps 
                SET 
                    target_user_id = ?, status = ?, notes = ?, updated_at = ?
                WHERE id = ?
                "#,
                responding_user_id,
                status_str,
                notes,
                now,
                id
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                r#"
                UPDATE shift_swaps 
                SET updated_at = ?
                WHERE id = ?
                "#,
                now,
                id
            )
            .execute(&self.pool)
            .await?;
        }

        // Fetch the updated swap
        match self.get_swap_by_id(id).await? {
            Some(swap) => Ok(swap),
            None => Err(anyhow::anyhow!("Swap not found after update")),
        }
    }

    /// Approve a swap request (managers/admins only)
    pub async fn approve_swap(
        &self,
        id: i64,
        approved_by: &str,
        notes: String,
    ) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();
        let status_str = ShiftSwapStatus::Approved.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, requesting_user_id, original_shift_id, target_user_id, target_shift_id,
                notes, swap_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            status_str,
            approved_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ShiftSwap {
            id: row
                .id
                .expect("ID should always be present in RETURNING clause"),
            requesting_user_id: row.requesting_user_id,
            original_shift_id: row.original_shift_id,
            target_user_id: row.target_user_id,
            target_shift_id: row.target_shift_id,
            notes: row.notes,
            swap_type: row.swap_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Deny a swap request (managers/admins only)
    pub async fn deny_swap(&self, id: i64, denied_by: &str, notes: String) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();
        let status_str = ShiftSwapStatus::Denied.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, requesting_user_id, original_shift_id, target_user_id, target_shift_id,
                notes, swap_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            status_str,
            denied_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ShiftSwap {
            id: row
                .id
                .expect("ID should always be present in RETURNING clause"),
            requesting_user_id: row.requesting_user_id,
            original_shift_id: row.original_shift_id,
            target_user_id: row.target_user_id,
            target_shift_id: row.target_shift_id,
            notes: row.notes,
            swap_type: row.swap_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Cancel a swap request
    pub async fn cancel_swap(&self, id: i64) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();
        let status_str = ShiftSwapStatus::Cancelled.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, requesting_user_id, original_shift_id, target_user_id, target_shift_id,
                notes, swap_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            status_str,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ShiftSwap {
            id: row
                .id
                .expect("ID should always be present in RETURNING clause"),
            requesting_user_id: row.requesting_user_id,
            original_shift_id: row.original_shift_id,
            target_user_id: row.target_user_id,
            target_shift_id: row.target_shift_id,
            notes: row.notes,
            swap_type: row.swap_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Complete a swap request
    pub async fn complete_swap(&self, id: i64) -> Result<ShiftSwap> {
        let now = Utc::now().naive_utc();
        let status_str = ShiftSwapStatus::Completed.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE shift_swaps 
            SET 
                status = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, requesting_user_id, original_shift_id, target_user_id, target_shift_id,
                notes, swap_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            status_str,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ShiftSwap {
            id: row
                .id
                .expect("ID should always be present in RETURNING clause"),
            requesting_user_id: row.requesting_user_id,
            original_shift_id: row.original_shift_id,
            target_user_id: row.target_user_id,
            target_shift_id: row.target_shift_id,
            notes: row.notes,
            swap_type: row.swap_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }
}
