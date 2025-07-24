use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    models::{
        ShiftSwap, ShiftSwapInput, ShiftSwapResponse, ShiftSwapResponseStatus, ShiftSwapStatus,
        ShiftSwapType, SwapResponse, SwapShift, SwapUser,
    },
    Result,
};

pub struct ShiftSwapRepository {
    pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct ShiftSwapResponseRow {
    id: Uuid,
    requesting_user_id: Uuid,
    original_shift_id: Uuid,
    target_user_id: Option<Uuid>,
    target_shift_id: Option<Uuid>,
    notes: Option<String>,
    swap_type: ShiftSwapType,
    status: ShiftSwapStatus,
    approved_by: Option<Uuid>,
    approval_notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    requesting_user_name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    department: String,
}

impl ShiftSwapRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new shift swap request
    pub async fn create_swap_request(&self, input: ShiftSwapInput) -> Result<ShiftSwap> {
        let now = Utc::now();

        // Determine initial status based on swap type
        let initial_status = match input.swap_type {
            ShiftSwapType::Open => ShiftSwapStatus::Open,
            ShiftSwapType::Targeted => ShiftSwapStatus::Pending,
        };

        let swap_type_str = input.swap_type.to_string();
        let status_str = initial_status.to_string();

        let shift_swap = sqlx::query_as::<_, ShiftSwap>(
            r#"
            INSERT INTO
                shift_swaps (
                    requesting_user_id,
                    original_shift_id,
                    target_user_id,
                    target_shift_id,
                    notes,
                    swap_type,
                    status,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id,
                requesting_user_id,
                original_shift_id,
                target_user_id,
                target_shift_id,
                notes,
                swap_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(input.requesting_user_id)
        .bind(input.original_shift_id)
        .bind(input.target_user_id)
        .bind(input.target_shift_id)
        .bind(input.notes)
        .bind(swap_type_str)
        .bind(status_str)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(shift_swap)
    }

    /// Get all swap requests with optional filtering
    pub async fn get_swap_requests_for_company(
        &self,
        user_id: Option<Uuid>,
        company_id: Uuid,
        status: Option<ShiftSwapStatus>,
        swap_type: Option<ShiftSwapType>,
    ) -> Result<Vec<ShiftSwap>> {
        // For simplicity, just get all and filter in memory for now
        let shift_swaps = sqlx::query_as::<_, ShiftSwap>(
            r#"
            SELECT
                id,
                requesting_user_id,
                original_shift_id,
                target_user_id,
                target_shift_id,
                notes,
                swap_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_swaps
            WHERE
                company_id = ?
            ORDER BY
                created_at DESC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut swaps = Vec::new();
        for swap in shift_swaps {
            // Apply filters
            if let Some(uid) = user_id {
                if swap.requesting_user_id != uid && swap.target_user_id.as_ref() != Some(&uid) {
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
    pub async fn get_swap_by_id(&self, id: Uuid) -> Result<Option<ShiftSwap>> {
        let shift_swap = sqlx::query_as::<_, ShiftSwap>(
            r#"
            SELECT
                id,
                requesting_user_id,
                original_shift_id,
                target_user_id,
                target_shift_id,
                notes,
                swap_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                shift_swaps
            WHERE
                id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(shift_swap)
    }

    /// Respond to a swap request (accept/decline)
    pub async fn respond_to_swap(
        &self,
        id: Uuid,
        responding_user_id: Uuid,
        accept: bool,
        notes: Option<String>,
    ) -> Result<ShiftSwap> {
        let now = Utc::now();
        let new_status = if accept {
            ShiftSwapStatus::Pending
        } else {
            ShiftSwapStatus::Open
        };
        let status_str = new_status.to_string();

        // Update the swap
        if accept {
            sqlx::query(
                r#"
                UPDATE
                    shift_swaps
                SET
                    target_user_id = ?,
                    status = ?,
                    notes = ?,
                    updated_at = ?
                WHERE
                    id = ?
                "#,
            )
            .bind(responding_user_id)
            .bind(status_str)
            .bind(notes)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE
                    shift_swaps
                SET
                    updated_at = ?
                WHERE
                    id = ?
                "#,
            )
            .bind(now)
            .bind(id)
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
        id: Uuid,
        approved_by: Uuid,
        notes: String,
    ) -> Result<ShiftSwap> {
        let now = Utc::now();
        let status_str = ShiftSwapStatus::Approved.to_string();

        let shift_swap = sqlx::query_as::<_, ShiftSwap>(
            r#"
            UPDATE
                shift_swaps
            SET
                status = ?,
                approved_by = ?,
                approval_notes = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                requesting_user_id,
                original_shift_id,
                target_user_id,
                target_shift_id,
                notes,
                swap_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(status_str)
        .bind(approved_by)
        .bind(notes)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(shift_swap)
    }

    /// Deny a swap request (managers/admins only)
    pub async fn deny_swap(&self, id: Uuid, denied_by: Uuid, notes: String) -> Result<ShiftSwap> {
        let now = Utc::now();
        let status_str = ShiftSwapStatus::Denied.to_string();

        let shift_swap = sqlx::query_as::<_, ShiftSwap>(
            r#"
            UPDATE
                shift_swaps
            SET
                status = ?,
                approved_by = ?,
                approval_notes = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                requesting_user_id,
                original_shift_id,
                target_user_id,
                target_shift_id,
                notes,
                swap_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(status_str)
        .bind(denied_by)
        .bind(notes)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(shift_swap)
    }

    /// Cancel a swap request
    pub async fn cancel_swap(&self, id: Uuid) -> Result<ShiftSwap> {
        let now = Utc::now();
        let status_str = ShiftSwapStatus::Cancelled.to_string();

        let shift_swap = sqlx::query_as::<_, ShiftSwap>(
            r#"
            UPDATE
                shift_swaps
            SET
                status = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                requesting_user_id,
                original_shift_id,
                target_user_id,
                target_shift_id,
                notes,
                swap_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(status_str)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(shift_swap)
    }

    /// Complete a swap request
    pub async fn complete_swap(&self, id: Uuid) -> Result<ShiftSwap> {
        let now = Utc::now();
        let status_str = ShiftSwapStatus::Completed.to_string();

        let shift_swap = sqlx::query_as::<_, ShiftSwap>(
            r#"
            UPDATE
                shift_swaps
            SET
                status = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                requesting_user_id,
                original_shift_id,
                target_user_id,
                target_shift_id,
                notes,
                swap_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(status_str)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(shift_swap)
    }

    /// Get all swap requests with full details (user and shift info) for frontend
    pub async fn get_swap_requests_with_details(
        &self,
        user_id: Option<Uuid>,
        company_id: Uuid,
        status: Option<ShiftSwapStatus>,
        swap_type: Option<ShiftSwapType>,
    ) -> Result<Vec<ShiftSwapResponse>> {
        let mut query = r#"
            SELECT
                ss.id,
                ss.requesting_user_id,
                ss.original_shift_id,
                ss.target_user_id,
                ss.target_shift_id,
                ss.notes,
                ss.swap_type,
                ss.status,
                ss.approved_by,
                ss.approval_notes,
                ss.created_at,
                ss.updated_at,
                u.name AS requesting_user_name,
                s.start_time,
                s.end_time,
                t.name AS department
            FROM
                shift_swaps ss
                JOIN users u ON ss.requesting_user_id = u.id
                JOIN shifts s ON ss.original_shift_id = s.id
                JOIN teams t ON s.team_id = t.id
        "#
        .to_string();

        let mut conditions = vec!["ss.company_id = ?"];
        let mut params = vec![company_id.to_string()];

        if let Some(uid) = user_id {
            conditions.push("(ss.requesting_user_id = ? OR ss.target_user_id = ?)");
            params.push(uid.to_string());
            params.push(uid.to_string());
        }

        if let Some(ref s) = status {
            conditions.push("ss.status = ?");
            params.push(s.to_string());
        }

        if let Some(ref st) = swap_type {
            conditions.push("ss.swap_type = ?");
            params.push(st.to_string());
        }

        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));

        query.push_str(" ORDER BY ss.created_at DESC");

        let mut prepared = sqlx::query_as::<_, ShiftSwapResponseRow>(&query);
        for (_, param) in params.into_iter().enumerate() {
            prepared = prepared.bind(param);
        }
        let rows = prepared.fetch_all(&self.pool).await?;

        let shift_swap_response = rows
            .into_iter()
            .map(|row| {
                ShiftSwapResponse {
                    id: row.id,
                    swap_type: row.swap_type.to_string(),
                    requested_by: SwapUser {
                        id: row.requesting_user_id,
                        name: row.requesting_user_name,
                        avatar: "".to_string(), // Will be filled by join query
                    },
                    original_shift: SwapShift {
                        id: row.original_shift_id,
                        start_time: row.start_time,
                        end_time: row.end_time,
                        department: row.department,
                    },
                    status: row.status.to_string(),
                    reason: row.notes.unwrap_or_default(),
                    request_date: row.created_at,
                    responses: None, // Will be filled by separate query if needed
                }
            })
            .collect();

        Ok(shift_swap_response)
    }

    /// Get a swap request by ID with full details
    pub async fn get_swap_by_id_with_details(&self, id: Uuid) -> Result<Option<ShiftSwapResponse>> {
        let row = sqlx::query_as::<_, ShiftSwapResponseRow>(
            r#"
            SELECT
                ss.id,
                ss.requesting_user_id,
                ss.original_shift_id,
                ss.target_user_id,
                ss.target_shift_id,
                ss.notes,
                ss.swap_type,
                ss.status,
                ss.approved_by,
                ss.approval_notes,
                ss.created_at,
                ss.updated_at,
                u.name AS requesting_user_name,
                s.start_time,
                s.end_time,
                t.name AS department
            FROM
                shift_swaps ss
                JOIN users u ON ss.requesting_user_id = u.id
                JOIN shifts s ON ss.original_shift_id = s.id
                JOIN teams t ON s.team_id = t.id
            WHERE
                ss.id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                Ok(Some(ShiftSwapResponse {
                    id: row.id,
                    swap_type: row.swap_type.to_string(),
                    requested_by: SwapUser {
                        id: row.requesting_user_id,
                        name: row.requesting_user_name,
                        avatar: "".to_string(), // Will be filled by join query
                    },
                    original_shift: SwapShift {
                        id: row.original_shift_id,
                        start_time: row.start_time,
                        end_time: row.end_time,
                        department: row.department,
                    },
                    status: row.status.to_string(),
                    reason: row.notes.unwrap_or_default(),
                    request_date: row.created_at,
                    responses: None, // Will be filled by separate query if needed
                }))
            }
            None => Ok(None),
        }
    }

    /// Get all responses for a specific swap request
    pub async fn get_swap_responses(&self, _swap_id: Uuid) -> Result<Vec<SwapResponse>> {
        // Note: This method needs to be updated when shift_swap_responses table schema is available
        // For now, return empty vec to avoid database errors
        Ok(Vec::new())
    }

    /// Create a response to a swap request
    pub async fn create_swap_response(
        &self,
        _swap_id: Uuid,
        responding_user_id: Uuid,
        status: ShiftSwapResponseStatus,
        _notes: Option<String>,
    ) -> Result<SwapResponse> {
        let _now = Utc::now();
        let status_str = status.to_string();

        // Note: This method needs to be updated when shift_swap_responses table schema is available
        // For now, return a placeholder response
        Ok(SwapResponse {
            id: Uuid::new_v4(),
            user: SwapUser {
                id: responding_user_id,
                name: "Unknown User".to_string(),
                avatar: "".to_string(),
            },
            status: status_str,
        })
    }

    /// Update a swap response status
    pub async fn update_swap_response_status(
        &self,
        response_id: Uuid,
        status: ShiftSwapResponseStatus,
    ) -> Result<SwapResponse> {
        let _now = Utc::now();
        let status_str = status.to_string();

        // Note: This method needs to be updated when shift_swap_responses table schema is available
        // For now, return a placeholder response
        Ok(SwapResponse {
            id: response_id,
            user: SwapUser {
                id: Uuid::new_v4(),
                name: "Unknown User".to_string(),
                avatar: "".to_string(),
            },
            status: status_str,
        })
    }
}
