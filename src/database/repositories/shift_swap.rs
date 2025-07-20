use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::database::{
    models::{
        ShiftSwap, ShiftSwapInput, ShiftSwapResponse, ShiftSwapResponseStatus, ShiftSwapStatus,
        ShiftSwapType, SwapResponse, SwapShift, SwapUser,
    },
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
            id: row.id.expect("Row ID should not be null"),
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

    /// Get all swap requests with full details (user and shift info) for frontend
    pub async fn get_swap_requests_with_details(
        &self,
        user_id: Option<&str>,
        status: Option<ShiftSwapStatus>,
        swap_type: Option<ShiftSwapType>,
    ) -> Result<Vec<ShiftSwapResponse>> {
        let mut query = r#"
            SELECT 
                ss.id, ss.requesting_user_id, ss.original_shift_id, ss.target_user_id, 
                ss.target_shift_id, ss.notes, ss.swap_type, ss.status, ss.approved_by, 
                ss.approval_notes, ss.created_at, ss.updated_at,
                u.name as requesting_user_name,
                s.start_time, s.end_time,
                t.name as department
            FROM shift_swaps ss
            JOIN users u ON ss.requesting_user_id = u.id
            JOIN shifts s ON ss.original_shift_id = s.id
            JOIN teams t ON s.team_id = t.id
        "#
        .to_string();

        let mut conditions = Vec::new();

        if let Some(uid) = user_id {
            conditions.push(format!(
                "(ss.requesting_user_id = '{}' OR ss.target_user_id = '{}')",
                uid, uid
            ));
        }

        if let Some(ref s) = status {
            conditions.push(format!("ss.status = '{}'", s.to_string()));
        }

        if let Some(ref st) = swap_type {
            conditions.push(format!("ss.swap_type = '{}'", st.to_string()));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY ss.created_at DESC");

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut swap_responses = Vec::new();

        for row in rows {
            let swap_id = row.get::<i64, _>("id");

            // Get responses for this swap
            let responses = self.get_swap_responses(swap_id).await.unwrap_or_default();

            let swap_response = ShiftSwapResponse {
                id: swap_id.to_string(),
                swap_type: row.get::<String, _>("swap_type"),
                requested_by: SwapUser {
                    id: row.get::<String, _>("requesting_user_id"),
                    name: row.get::<String, _>("requesting_user_name"),
                    avatar: "".to_string(), // Default empty avatar for now
                },
                original_shift: SwapShift {
                    id: row.get::<i64, _>("original_shift_id").to_string(),
                    start_time: row.get("start_time"),
                    end_time: row.get("end_time"),
                    department: row.get::<String, _>("department"),
                },
                status: row.get::<String, _>("status"),
                reason: row.get::<Option<String>, _>("notes").unwrap_or_default(),
                request_date: row.get("created_at"),
                responses: if responses.is_empty() {
                    None
                } else {
                    Some(responses)
                },
            };

            swap_responses.push(swap_response);
        }

        Ok(swap_responses)
    }

    /// Get a swap request by ID with full details
    pub async fn get_swap_by_id_with_details(&self, id: i64) -> Result<Option<ShiftSwapResponse>> {
        let row = sqlx::query!(
            r#"
            SELECT 
                ss.id, ss.requesting_user_id, ss.original_shift_id, ss.target_user_id, 
                ss.target_shift_id, ss.notes, ss.swap_type, ss.status, ss.approved_by, 
                ss.approval_notes, ss.created_at, ss.updated_at,
                u.name as requesting_user_name,
                s.start_time, s.end_time,
                t.name as department
            FROM shift_swaps ss
            JOIN users u ON ss.requesting_user_id = u.id
            JOIN shifts s ON ss.original_shift_id = s.id
            JOIN teams t ON s.team_id = t.id
            WHERE ss.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                // Get responses for this swap
                let responses = self.get_swap_responses(id).await.unwrap_or_default();

                Ok(Some(ShiftSwapResponse {
                    id: row.id.to_string(),
                    swap_type: row.swap_type,
                    requested_by: SwapUser {
                        id: row.requesting_user_id,
                        name: row.requesting_user_name,
                        avatar: "".to_string(), // Default empty avatar for now
                    },
                    original_shift: SwapShift {
                        id: row.original_shift_id.to_string(),
                        start_time: row.start_time,
                        end_time: row.end_time,
                        department: row.department,
                    },
                    status: row.status,
                    reason: row.notes.unwrap_or_default(),
                    request_date: row.created_at.unwrap(),
                    responses: if responses.is_empty() {
                        None
                    } else {
                        Some(responses)
                    },
                }))
            }
            None => Ok(None),
        }
    }

    /// Get all responses for a specific swap request
    pub async fn get_swap_responses(&self, swap_id: i64) -> Result<Vec<SwapResponse>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                ssr.id, ssr.swap_id, ssr.responding_user_id, ssr.status, ssr.notes,
                ssr.created_at, ssr.updated_at,
                u.name as responding_user_name
            FROM shift_swap_responses ssr
            JOIN users u ON ssr.responding_user_id = u.id
            WHERE ssr.swap_id = ?
            ORDER BY ssr.created_at ASC
            "#,
            swap_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut responses = Vec::new();
        for row in rows {
            let response = SwapResponse {
                id: row.id.expect("Response ID should not be null").to_string(),
                user: SwapUser {
                    id: row.responding_user_id,
                    name: row.responding_user_name,
                    avatar: "".to_string(), // Default empty avatar for now
                },
                status: row.status,
            };
            responses.push(response);
        }

        Ok(responses)
    }

    /// Create a response to a swap request
    pub async fn create_swap_response(
        &self,
        swap_id: i64,
        responding_user_id: &str,
        status: ShiftSwapResponseStatus,
        notes: Option<String>,
    ) -> Result<SwapResponse> {
        let now = Utc::now().naive_utc();
        let status_str = status.to_string();

        let row = sqlx::query!(
            r#"
            INSERT INTO shift_swap_responses (
                swap_id, responding_user_id, status, notes, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, swap_id, responding_user_id, status, notes, created_at, updated_at
            "#,
            swap_id,
            responding_user_id,
            status_str,
            notes,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        // Get user details
        let user_row = sqlx::query!("SELECT name FROM users WHERE id = ?", responding_user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(SwapResponse {
            id: row.id.expect("Response ID should not be null").to_string(),
            user: SwapUser {
                id: responding_user_id.to_string(),
                name: user_row.name,
                avatar: "".to_string(),
            },
            status: row.status,
        })
    }

    /// Update a swap response status
    pub async fn update_swap_response_status(
        &self,
        response_id: i64,
        status: ShiftSwapResponseStatus,
    ) -> Result<SwapResponse> {
        let now = Utc::now().naive_utc();
        let status_str = status.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE shift_swap_responses 
            SET status = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, swap_id, responding_user_id, status, notes, created_at, updated_at
            "#,
            status_str,
            now,
            response_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Get user details
        let user_row = sqlx::query!(
            "SELECT name FROM users WHERE id = ?",
            row.responding_user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SwapResponse {
            id: row.id.to_string(),
            user: SwapUser {
                id: row.responding_user_id,
                name: user_row.name,
                avatar: "".to_string(),
            },
            status: row.status,
        })
    }
}
