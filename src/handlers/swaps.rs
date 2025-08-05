use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::{
    models::{activity::Action, ShiftSwapInput, ShiftSwapResponseStatus, ShiftSwapStatus},
    repositories::{company as company_repo, shift_swap as shift_swap_repo},
};
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::{activity_logger, user_context::extract_context};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuery {
    pub requesting_user_id: Option<Uuid>,
    pub target_user_id: Option<Uuid>,
    pub status: Option<ShiftSwapStatus>,
    pub original_shift_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponseRequest {
    pub target_shift_id: Option<Uuid>,
    pub decision: ShiftSwapResponseStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}

/// Create a new shift swap request
pub async fn create_swap_request(
    input: web::Json<ShiftSwapInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    // Users can only create swap requests for themselves unless they're managers/admins
    let request_input = input.into_inner();
    let original_shift_id = request_input.original_shift_id;
    let target_user_id = request_input.target_user_id.clone();
    let requesting_user_id = request_input.requesting_user_id.clone();
    let company_id = user_context.strict_company_id()?;

    user_context.requires_same_user(requesting_user_id)?;

    let swap_request = shift_swap_repo::create_swap_request(request_input)
        .await
        .map_err(|e| {
            log::error!("Failed to create swap request: {}", e);
            AppError::DatabaseError(e)
        })?; // Handle error and return early if creation fails
    {
        // Log the swap creation activity
        let metadata = activity_logger::metadata(vec![
            ("original_shift_id", original_shift_id.to_string()),
            ("requesting_user_id", requesting_user_id.to_string()),
            (
                "target_user_id",
                target_user_id.map_or("None".to_string(), |id| id.to_string()),
            ),
        ]);

        if let Err(e) = activity_logger::log_shift_swap_activity(
            company_id,
            Some(user_context.user_id()),
            swap_request.id,
            Action::CREATED,
            format!(
                "Shift swap request created by user {} for shift {}",
                requesting_user_id, original_shift_id
            ),
            Some(metadata),
            &req,
        )
        .await
        {
            log::warn!("Failed to log shift swap creation activity: {}", e);
        }
    }

    Ok(ApiResponse::created(swap_request))
}

/// Get shift swap requests with optional filtering
pub async fn get_swap_requests(
    query: web::Query<SwapQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();
    let company_id = user_context.strict_company_id()?;
    let requesting_user_id = query.requesting_user_id;
    let status_filter = query.status.clone();

    if requesting_user_id.is_none() {
        user_context.requires_manager_or("Manager access required to view all swap requests")?;
    } else {
        user_context.requires_same_user(requesting_user_id.unwrap_or(user_id))?;
    }

    let requests = shift_swap_repo::get_swap_requests_with_details(
        requesting_user_id,
        company_id, // Add company_id
        status_filter,
        None, // swap_type
    )
    .await
    .map_err(|e| {
        log::error!("Failed to fetch swap requests: {}", e);
        AppError::DatabaseError(e)
    })?;

    Ok(ApiResponse::success(requests))
}

/// Get a specific shift swap request by ID
pub async fn get_swap_request(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let swap_id = path.into_inner();

    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch swap request: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Swap request not found for ID: {}", swap_id);
            AppError::NotFound("Swap request not found".to_string())
        })?;

    if !user_context.is_manager_or_admin() {
        let is_involved = swap_request.requesting_user_id == user_context.user.id
            || swap_request.target_user_id.as_ref() == Some(&user_context.user.id);
        if !is_involved {
            return Err(AppError::PermissionDenied(
                "Cannot view other users' swap requests".to_string(),
            )
            .into());
        }
    }

    Ok(ApiResponse::success(swap_request))
}

/// Respond to a shift swap request (for targeted swaps)
pub async fn respond_to_swap(
    path: web::Path<Uuid>,
    response: web::Json<SwapResponseRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let swap_id = path.into_inner();

    // First get the swap request to check permissions
    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch swap request for response: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Swap request not found for ID: {}", swap_id);
            AppError::NotFound("Swap request not found".to_string())
        })?;

    // Only the target user can respond to a targeted swap
    if swap_request.target_user_id.as_ref() != Some(&user_context.user.id) {
        return Err(AppError::PermissionDenied(
            "Only the target user can respond to this swap request".to_string(),
        )
        .into());
    }

    // Can only respond to open swaps
    if swap_request.status != ShiftSwapStatus::Open {
        return Err(AppError::BadRequest("Can only respond to open swaps".to_string()).into());
    }

    let original_shift_id = swap_request.original_shift_id;
    let requesting_user_id = swap_request.requesting_user_id;
    let decision = response.decision.clone();
    let notes = response.notes.clone();

    // Note: Type mismatch issues - using workarounds for now
    let updated_swap = shift_swap_repo::create_swap_response(
        swap_id,
        user_context.user.id,
        decision.clone(),
        notes.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Failed to respond to swap request: {}", e);
        AppError::DatabaseError(e)
    })?;

    if let Ok(Some(company)) =
        company_repo::get_primary_company_for_user(user_context.user.id).await
    {
        let metadata = activity_logger::metadata(vec![
            ("original_shift_id", original_shift_id.to_string()),
            ("requesting_user_id", requesting_user_id.to_string()),
            (
                "target_shift_id",
                response
                    .target_shift_id
                    .map_or("None".to_string(), |id| id.to_string()),
            ),
            ("notes", response.notes.clone().unwrap_or_default()),
        ]);

        if let Err(e) = activity_logger::log_shift_swap_activity(
            company.id,
            Some(user_context.user.id),
            swap_id, // Note: Type mismatch with i64 expected
            Action::UPDATED,
            format!(
                "User {} responded to swap request from {} with decision {:?}",
                user_context.user.id, requesting_user_id, decision
            ),
            Some(metadata),
            &req,
        )
        .await
        {
            log::warn!("Failed to log swap response activity: {}", e);
        }
    }

    Ok(ApiResponse::success(updated_swap))
}

/// Approve a shift swap request (managers/admins only)
pub async fn approve_swap_request(
    path: web::Path<Uuid>,
    approval: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let swap_id = path.into_inner();

    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching swap request for approval: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Swap request not found for ID: {}", swap_id);
            AppError::NotFound("Swap request not found".to_string())
        })?;

    let shift_swap = shift_swap_repo::approve_swap(
        swap_id,
        user_context.user.id,
        approval.notes.clone().unwrap_or_default(),
    )
    .await
    .map_err(|e| {
        log::error!("Error approving swap request: {}", e);
        AppError::DatabaseError(e)
    })?;

    // Log swap approval activity
    let company_id = user_context.strict_company_id()?;

    let metadata = activity_logger::metadata(vec![
        (
            "original_shift_id",
            swap_request.original_shift_id.to_string(),
        ),
        (
            "requesting_user_id",
            swap_request.requesting_user_id.to_string(),
        ),
        (
            "target_user_id",
            swap_request
                .target_user_id
                .map_or("None".to_string(), |id| id.to_string()),
        ),
        ("approval_notes", approval.notes.clone().unwrap_or_default()),
    ]);

    if let Err(e) = activity_logger::log_shift_swap_activity(
        company_id,
        Some(user_context.user.id),
        swap_id, // Note: Type mismatch with expected Uuid vs i64
        Action::APPROVED,
        format!(
            "Shift swap request approved for user {}",
            swap_request.requesting_user_id
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log swap approval activity: {}", e);
    }

    Ok(ApiResponse::success(shift_swap))
}

/// Deny a shift swap request (managers/admins only)
pub async fn deny_swap_request(
    path: web::Path<Uuid>,
    denial: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let swap_id = path.into_inner();

    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching swap request for denial: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Swap request not found for ID: {}", swap_id);
            AppError::NotFound("Swap request not found".to_string())
        })?;

    let shift_swap = shift_swap_repo::deny_swap(
        swap_id,
        user_context.user.id,
        denial.notes.clone().unwrap_or_default(),
    )
    .await
    .map_err(|e| {
        log::error!("Error denying swap request: {}", e);
        AppError::DatabaseError(e)
    })?;

    let company_id = user_context.strict_company_id()?;
    // Log swap denial activity

    let metadata = activity_logger::metadata(vec![
        (
            "original_shift_id",
            swap_request.original_shift_id.to_string(),
        ),
        (
            "requesting_user_id",
            swap_request.requesting_user_id.to_string(),
        ),
        (
            "target_user_id",
            swap_request
                .target_user_id
                .map_or("None".to_string(), |id| id.to_string()),
        ),
        ("denial_notes", denial.notes.clone().unwrap_or_default()),
    ]);

    if let Err(e) = activity_logger::log_shift_swap_activity(
        company_id,
        Some(user_context.user.id),
        swap_id, // Note: Type mismatch with expected Uuid vs i64
        Action::REJECTED,
        format!(
            "Shift swap request denied for user {}",
            swap_request.requesting_user_id
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log swap denial activity: {}", e);
    }

    Ok(ApiResponse::success(shift_swap))
}
