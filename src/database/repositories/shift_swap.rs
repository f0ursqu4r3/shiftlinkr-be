use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{
    get_pool,
    models::{
        Shift, ShiftSwap, ShiftSwapInput, ShiftSwapResponse, ShiftSwapResponseStatus,
        ShiftSwapStatus, ShiftSwapType, UserInfo,
    },
    utils::sql,
};

#[derive(sqlx::FromRow)]
struct ShiftSwapDetailRaw {
    id: Uuid,
    swap_type: ShiftSwapType,
    requesting_user_id: Uuid,
    status: ShiftSwapStatus,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    // Additional fields for shift details
    shift_id: Uuid,
    shift_company_id: Uuid,
    shift_title: String,
    shift_description: Option<String>,
    shift_location_id: Option<Uuid>,
    shift_team_id: Option<Uuid>,
    shift_start_time: DateTime<Utc>,
    shift_end_time: DateTime<Utc>,
    shift_min_duration_minutes: Option<i32>,
    shift_max_duration_minutes: Option<i32>,
    shift_max_people: Option<i32>,
    shift_status: String,
    shift_created_at: DateTime<Utc>,
    shift_updated_at: DateTime<Utc>,
    // Additional fields for user details
    requesting_user_email: String,
    requesting_user_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ShiftSwapDetail {
    pub id: Uuid,
    pub swap_type: ShiftSwapType,
    pub requested_by: UserInfo,
    pub original_shift: Shift,
    pub status: ShiftSwapStatus,
    pub reason: String,
    pub request_date: DateTime<Utc>,
    pub responses: Vec<ShiftSwapResponse>,
}

/// Create a new shift swap request
pub async fn create_swap_request(input: ShiftSwapInput) -> Result<ShiftSwap> {
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
                ($1, $2, $3, $4, $5, $6, $7, $8, $9)
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
    .fetch_one(get_pool())
    .await?;

    Ok(shift_swap)
}

/// Get all swap requests with optional filtering
pub async fn get_swap_requests_for_company(
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
                company_id = $1
            ORDER BY
                created_at DESC
            "#,
    )
    .bind(company_id)
    .fetch_all(get_pool())
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
pub async fn find_swap_request_by_id(id: Uuid) -> Result<Option<ShiftSwap>> {
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
                id = $1
            "#,
    )
    .bind(id)
    .fetch_optional(get_pool())
    .await?;

    Ok(shift_swap)
}

/// Approve a swap request (managers/admins only)
pub async fn approve_swap(id: Uuid, approved_by: Uuid, notes: String) -> Result<ShiftSwap> {
    let now = Utc::now();
    let status_str = ShiftSwapStatus::Approved.to_string();

    let shift_swap = sqlx::query_as::<_, ShiftSwap>(
        r#"
            UPDATE
                shift_swaps
            SET
                status = $1,
                approved_by = $2,
                approval_notes = $3,
                updated_at = $4
            WHERE
                id = $5
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
    .fetch_one(get_pool())
    .await?;

    Ok(shift_swap)
}

/// Deny a swap request (managers/admins only)
pub async fn deny_swap(id: Uuid, denied_by: Uuid, notes: String) -> Result<ShiftSwap> {
    let now = Utc::now();
    let status_str = ShiftSwapStatus::Denied.to_string();

    let shift_swap = sqlx::query_as::<_, ShiftSwap>(
        r#"
            UPDATE
                shift_swaps
            SET
                status = $1,
                approved_by = $2,
                approval_notes = $3,
                updated_at = $4
            WHERE
                id = $5
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
    .fetch_one(get_pool())
    .await?;

    Ok(shift_swap)
}

/// Cancel a swap request
pub async fn cancel_swap(id: Uuid) -> Result<ShiftSwap> {
    let now = Utc::now();
    let status_str = ShiftSwapStatus::Cancelled.to_string();

    let shift_swap = sqlx::query_as::<_, ShiftSwap>(
        r#"
            UPDATE
                shift_swaps
            SET
                status = $1,
                updated_at = $2
            WHERE
                id = $3
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
    .fetch_one(get_pool())
    .await?;

    Ok(shift_swap)
}

/// Complete a swap request
pub async fn complete_swap(id: Uuid) -> Result<ShiftSwap> {
    let now = Utc::now();
    let status_str = ShiftSwapStatus::Completed.to_string();

    let shift_swap = sqlx::query_as::<_, ShiftSwap>(
        r#"
            UPDATE
                shift_swaps
            SET
                status = $1,
                updated_at = $2
            WHERE
                id = $3
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
    .fetch_one(get_pool())
    .await?;

    Ok(shift_swap)
}

/// Get all swap requests with full details (user and shift info) for frontend
pub async fn get_swap_requests_with_details(
    user_id: Option<Uuid>,
    company_id: Uuid,
    status: Option<ShiftSwapStatus>,
    swap_type: Option<ShiftSwapType>,
) -> Result<Vec<ShiftSwapDetail>> {
    let mut query = r#"
            SELECT
                ss.id,
                ss.swap_type,
                ss.requesting_user_id,
                ss.original_shift_id,
                ss.status,
                ss.notes,
                ss.created_at,
                -- Additional fields for shift details
                s.id AS shift_id,
                s.company_id AS shift_company_id,
                s.title AS shift_title,
                s.description AS shift_description,
                s.location_id AS shift_location_id,
                s.team_id AS shift_team_id,
                s.start_time AS shift_start_time,
                s.end_time AS shift_end_time,
                s.min_duration_minutes AS shift_min_duration_minutes,
                s.max_duration_minutes AS shift_max_duration_minutes,
                s.max_people AS shift_max_people,
                s.status AS shift_status,
                s.created_at AS shift_created_at,
                s.updated_at AS shift_updated_at,
                -- Additional fields for user details
                u.id AS requesting_user_id,
                u.email AS requesting_user_email,
                u.name AS requesting_user_name

            FROM
                shift_swaps ss
                JOIN users u ON u.id = ss.requesting_user_id
                JOIN shifts s ON s.id = ss.original_shift_id
        "#
    .to_string();

    let mut conditions = vec!["s.company_id = ?"];
    let mut params = vec![company_id.to_string()];

    if let Some(status_val) = status {
        conditions.push("ss.status = ?");
        params.push(status_val.to_string());
    }
    if let Some(swap_type_val) = swap_type {
        conditions.push("ss.swap_type = ?");
        params.push(swap_type_val.to_string());
    }
    if let Some(uid) = user_id {
        conditions.push("(ss.requesting_user_id = ? OR ss.target_user_id = ?)");
        params.push(uid.to_string());
        params.push(uid.to_string());
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    // Build base query with company filter
    query.push_str(" ORDER BY ss.created_at DESC");

    let sql_query = sql(&query);
    let mut prepared = sqlx::query_as::<_, ShiftSwapDetailRaw>(&sql_query);
    for param in params {
        prepared = prepared.bind(param);
    }

    let rows = prepared.fetch_all(get_pool()).await?;

    let shift_swap_response = rows
        .into_iter()
        .map(|row| ShiftSwapDetail {
            id: row.id,
            swap_type: row.swap_type,
            requested_by: UserInfo {
                id: row.requesting_user_id,
                name: row.requesting_user_name,
                email: row.requesting_user_email,
            },
            original_shift: Shift {
                id: row.shift_id,
                company_id: row.shift_company_id,
                title: row.shift_title,
                description: row.shift_description,
                location_id: row.shift_location_id.unwrap_or_default(),
                team_id: row.shift_team_id,
                start_time: row.shift_start_time,
                end_time: row.shift_end_time,
                min_duration_minutes: row.shift_min_duration_minutes,
                max_duration_minutes: row.shift_max_duration_minutes,
                max_people: row.shift_max_people,
                status: row.shift_status.parse().unwrap_or_default(),
                created_at: row.shift_created_at,
                updated_at: row.shift_updated_at,
            },
            status: row.status,
            reason: row.notes.unwrap_or_default(),
            request_date: row.created_at,
            responses: vec![],
        })
        .collect();

    Ok(shift_swap_response)
}

/// Get a swap request by ID with full details
pub async fn get_swap_by_id_with_details(id: Uuid) -> Result<ShiftSwapDetail> {
    let row = sqlx::query_as::<_, ShiftSwapDetailRaw>(&sql(r#"
            SELECT
                ss.id,
                ss.swap_type,
                ss.requesting_user_id,
                ss.original_shift_id,
                ss.status,
                ss.notes,
                ss.created_at,
                -- Additional fields for shift details
                s.id AS shift_id,
                s.company_id AS shift_company_id,
                s.title AS shift_title,
                s.description AS shift_description,
                s.location_id AS shift_location_id,
                s.team_id AS shift_team_id,
                s.start_time AS shift_start_time,
                s.end_time AS shift_end_time,
                s.min_duration_minutes AS shift_min_duration_minutes,
                s.max_duration_minutes AS shift_max_duration_minutes,
                s.max_people AS shift_max_people,
                s.status AS shift_status,
                s.created_at AS shift_created_at,
                s.updated_at AS shift_updated_at,
                -- Additional fields for user details
                u.id AS requesting_user_id,
                u.email AS requesting_user_email,
                u.name AS requesting_user_name

            FROM
                shift_swaps ss
                JOIN users u ON u.id = ss.requesting_user_id
                JOIN shifts s ON s.id = ss.original_shift_id
            WHERE
                ss.id = $1
        "#))
    .bind(id)
    .fetch_one(get_pool())
    .await?;

    let mut shift_swap_detail = ShiftSwapDetail {
        id: row.id,
        swap_type: row.swap_type,
        requested_by: UserInfo {
            id: row.requesting_user_id,
            name: row.requesting_user_name,
            email: "".to_string(), // Will be filled by join query
        },
        original_shift: Shift {
            id: row.shift_id,
            company_id: row.shift_company_id,
            title: row.shift_title,
            description: row.shift_description,
            location_id: row.shift_location_id.unwrap_or_default(),
            team_id: row.shift_team_id,
            start_time: row.shift_start_time,
            end_time: row.shift_end_time,
            min_duration_minutes: row.shift_min_duration_minutes,
            max_duration_minutes: row.shift_max_duration_minutes,
            max_people: row.shift_max_people,
            status: row.shift_status.parse().unwrap_or_default(),
            created_at: row.shift_created_at,
            updated_at: row.shift_updated_at,
        },
        status: row.status,
        reason: row.notes.unwrap_or_default(),
        request_date: row.created_at,
        responses: vec![],
    };
    let responses = get_swap_responses(row.id).await?;
    shift_swap_detail.responses = responses;
    Ok(shift_swap_detail)
}

/// Get all responses for a specific swap request
pub async fn get_swap_responses(swap_id: Uuid) -> Result<Vec<ShiftSwapResponse>> {
    let responses = sqlx::query_as::<_, ShiftSwapResponse>(&sql(r#"
            SELECT
                id,
                swap_id,
                responding_user_id,
                response_type,
                notes,
                created_at
            FROM
                shift_swap_responses
            WHERE
                swap_id = ?
        "#))
    .bind(swap_id)
    .fetch_all(get_pool())
    .await?;

    Ok(responses)
}

pub async fn get_swap_response_for_user(
    swap_id: Uuid,
    user_id: Uuid,
) -> Result<Option<ShiftSwapResponse>> {
    let response = sqlx::query_as::<_, ShiftSwapResponse>(&sql(r#"
            SELECT
                id,
                swap_id,
                responding_user_id,
                response_type,
                notes,
                created_at
            FROM
                shift_swap_responses
            WHERE
                swap_id = ? AND responding_user_id = ?
        "#))
    .bind(swap_id)
    .bind(user_id)
    .fetch_optional(get_pool())
    .await?;

    Ok(response)
}

pub async fn create_swap_response(
    swap_id: Uuid,
    user_id: Uuid,
    response_type: ShiftSwapResponseStatus,
    notes: Option<String>,
) -> Result<ShiftSwapResponse> {
    let now = Utc::now();

    let shift_swap_response = sqlx::query_as::<_, ShiftSwapResponse>(&sql(r#"
            INSERT INTO
                shift_swap_responses (swap_id, responding_user_id, response_type, notes, created_at, updated_at)
            VALUES
                (?, ?, ?, ?, ?, ?)
            ON CONFLICT (swap_id, responding_user_id) DO UPDATE
            SET
                response_type = ?,
                notes = ?,
                updated_at = ?
            RETURNING
                id,
                swap_id,
                responding_user_id,
                response_type,
                notes,
                created_at
        "#))
        .bind(swap_id)
        .bind(user_id)
        .bind(response_type.clone())
        .bind(notes.clone())
        .bind(now)
        .bind(now)
        .bind(response_type)
        .bind(notes)
        .bind(now)
        .fetch_one(get_pool())
        .await?;

    Ok(shift_swap_response)
}

/// Update a swap response status
pub async fn update_swap_response_status(
    response_id: Uuid,
    response_type: ShiftSwapResponseStatus,
    notes: Option<String>,
) -> Result<ShiftSwapResponse> {
    let now = Utc::now();

    let shift_swap_response = sqlx::query_as::<_, ShiftSwapResponse>(&sql(r#"
            UPDATE
                shift_swap_responses
            SET
                response_type = ?,
                notes = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                swap_id,
                responding_user_id,
                response_type,
                notes,
                created_at
        "#))
    .bind(response_type)
    .bind(notes)
    .bind(now)
    .bind(response_id)
    .fetch_one(get_pool())
    .await?;

    Ok(shift_swap_response)
}
