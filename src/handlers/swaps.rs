use actix_web::{HttpResponse, Result, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        models::{ShiftSwapInput, ShiftSwapResponseStatus, ShiftSwapStatus, activity::Action},
        repositories::shift_swap as shift_swap_repo,
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::request_info::RequestInfo,
    services::{activity_logger, user_context::UserContext},
};

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
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    // Users can only create swap requests for themselves unless they're managers/admins
    let request_input = input.into_inner();
    let original_shift_id = request_input.original_shift_id;
    let target_user_id = request_input.target_user_id.clone();
    let requesting_user_id = request_input.requesting_user_id.clone();
    let company_id = ctx.strict_company_id()?;

    ctx.requires_same_user(requesting_user_id)?;

    let swap_request = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let swap_request = shift_swap_repo::create_swap_request(tx, request_input).await?;

            // Log the swap creation activity
            let metadata = activity_logger::metadata(vec![
                ("original_shift_id", original_shift_id.to_string()),
                ("requesting_user_id", requesting_user_id.to_string()),
                (
                    "target_user_id",
                    target_user_id.map_or("None".to_string(), |id| id.to_string()),
                ),
            ]);

            activity_logger::log_shift_swap_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                swap_request.id,
                &Action::CREATED,
                "Shift swap request created".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(swap_request)
        })
    })
    .await?;

    // Invalidate cached GETs in swaps scope
    cache.bump();
    Ok(ApiResponse::created(swap_request))
}

/// Get shift swap requests with optional filtering
pub async fn get_swap_requests(
    query: web::Query<SwapQuery>,
    ctx: UserContext,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let requesting_user_id = query.requesting_user_id;
    let status_filter = query.status.clone();

    if requesting_user_id.is_none() {
        ctx.requires_manager_or("Manager access required to view all swap requests")?;
    } else {
        ctx.requires_same_user(requesting_user_id.unwrap_or(user_id))?;
    }

    let requests = shift_swap_repo::get_swap_requests_with_details(
        requesting_user_id,
        company_id,
        status_filter,
        None, // swap_type
    )
    .await
    .map_err(AppError::from)?;

    Ok(ApiResponse::success(requests))
}

/// Get a specific shift swap request by ID
pub async fn get_swap_request(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let swap_id = path.into_inner();

    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Swap request not found".to_string()))?;

    if !ctx.is_manager_or_admin() {
        let is_involved = swap_request.requesting_user_id == ctx.user.id
            || swap_request.target_user_id.as_ref() == Some(&ctx.user.id);
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
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let swap_id = path.into_inner();

    // First get the swap request to check permissions
    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Swap request not found".to_string()))?;

    // Only the target user can respond to a targeted swap
    if swap_request.target_user_id.as_ref() != Some(&ctx.user.id) {
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
    let company_id = ctx.strict_company_id()?;

    let updated_swap = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let updated_swap = shift_swap_repo::create_swap_response(
                tx,
                swap_id,
                ctx.user.id,
                decision.clone(),
                notes.clone(),
            )
            .await?;

            // Log the activity
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

            activity_logger::log_shift_swap_activity(
                tx,
                company_id,
                Some(ctx.user.id),
                swap_id,
                &Action::UPDATED,
                "User responded to swap request".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(updated_swap)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(updated_swap))
}

/// Approve a shift swap request (managers/admins only)
pub async fn approve_swap_request(
    path: web::Path<Uuid>,
    approval: web::Json<ApprovalRequest>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let swap_id = path.into_inner();

    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Swap request not found".to_string()))?;

    let company_id = ctx.strict_company_id()?;

    let shift_swap = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let shift_swap = shift_swap_repo::approve_swap(
                tx,
                swap_id,
                ctx.user.id,
                approval.notes.clone().unwrap_or_default(),
            )
            .await?;

            // Log swap approval activity
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

            activity_logger::log_shift_swap_activity(
                tx,
                company_id,
                Some(ctx.user.id),
                swap_id,
                &Action::APPROVED,
                "Shift swap request approved".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(shift_swap)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(shift_swap))
}

/// Deny a shift swap request (managers/admins only)
pub async fn deny_swap_request(
    path: web::Path<Uuid>,
    denial: web::Json<ApprovalRequest>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let swap_id = path.into_inner();

    let swap_request = shift_swap_repo::find_swap_request_by_id(swap_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Swap request not found".to_string()))?;

    let company_id = ctx.strict_company_id()?;

    let shift_swap = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let shift_swap = shift_swap_repo::deny_swap(
                tx,
                swap_id,
                ctx.user.id,
                denial.notes.clone().unwrap_or_default(),
            )
            .await?;

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

            activity_logger::log_shift_swap_activity(
                tx,
                company_id,
                Some(ctx.user.id),
                swap_id,
                &Action::REJECTED,
                "Shift swap request denied".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(shift_swap)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(shift_swap))
}
