use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;

use crate::services::auth::Claims;
use crate::database::models::{ShiftSwapInput, ShiftSwapStatus};
use crate::database::repositories::shift_swap_repository::ShiftSwapRepository;
use crate::handlers::admin::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct SwapQuery {
    pub requesting_user_id: Option<String>,
    pub target_user_id: Option<String>,
    pub status: Option<String>,
    pub original_shift_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SwapResponseRequest {
    pub target_shift_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}

/// Create a new shift swap request
pub async fn create_swap_request(
    claims: Claims,
    repo: web::Data<ShiftSwapRepository>,
    input: web::Json<ShiftSwapInput>,
) -> Result<HttpResponse> {
    // Users can only create swap requests for themselves unless they're managers/admins
    let mut request_input = input.into_inner();

    if !claims.is_admin() && !claims.is_manager() && request_input.requesting_user_id != claims.sub
    {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Can only create swap requests for yourself",
        )));
    }

    // If employee, force requesting_user_id to be their own ID
    if !claims.is_admin() && !claims.is_manager() {
        request_input.requesting_user_id = claims.sub;
    }

    match repo.create_swap_request(request_input).await {
        Ok(swap_request) => Ok(HttpResponse::Created().json(ApiResponse::success(swap_request))),
        Err(err) => {
            log::error!("Error creating swap request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create swap request")))
        }
    }
}

/// Get shift swap requests with optional filtering
pub async fn get_swap_requests(
    claims: Claims,
    repo: web::Data<ShiftSwapRepository>,
    query: web::Query<SwapQuery>,
) -> Result<HttpResponse> {
    // Employees can only see swaps related to them (as requester or target)
    let requesting_user_id = if !claims.is_admin() && !claims.is_manager() {
        Some(claims.sub.as_str())
    } else {
        query.requesting_user_id.as_deref()
    };

    // Convert status string to enum if provided
    let status_filter = if let Some(status_str) = &query.status {
        match status_str.parse::<ShiftSwapStatus>() {
            Ok(status) => Some(status),
            Err(_) => {
                return Ok(
                    HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid status"))
                )
            }
        }
    } else {
        None
    };

    match repo
        .get_swap_requests(requesting_user_id, status_filter, None)
        .await
    {
        Ok(requests) => {
            // For employees, filter to only include swaps they're involved in
            let filtered_requests = if !claims.is_admin() && !claims.is_manager() {
                requests
                    .into_iter()
                    .filter(|swap| {
                        swap.requesting_user_id == claims.sub
                            || swap.target_user_id.as_ref() == Some(&claims.sub)
                    })
                    .collect()
            } else {
                requests
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(filtered_requests)))
        }
        Err(err) => {
            log::error!("Error fetching swap requests: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch swap requests")))
        }
    }
}

/// Get a specific shift swap request by ID
pub async fn get_swap_request(
    claims: Claims,
    repo: web::Data<ShiftSwapRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let swap_id = path.into_inner();

    match repo.get_swap_by_id(swap_id).await {
        Ok(Some(swap_request)) => {
            // Check if user has permission to view this swap
            if !claims.is_admin() && !claims.is_manager() {
                let is_involved = swap_request.requesting_user_id == claims.sub
                    || swap_request.target_user_id.as_ref() == Some(&claims.sub);
                if !is_involved {
                    return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                        "Cannot view other users' swap requests",
                    )));
                }
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(swap_request)))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Swap request not found")))
        }
        Err(err) => {
            log::error!("Error fetching swap request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch swap request")))
        }
    }
}

/// Respond to a shift swap request (for targeted swaps)
pub async fn respond_to_swap(
    claims: Claims,
    repo: web::Data<ShiftSwapRepository>,
    path: web::Path<i64>,
    response: web::Json<SwapResponseRequest>,
) -> Result<HttpResponse> {
    let swap_id = path.into_inner();

    // First get the swap request to check permissions
    match repo.get_swap_by_id(swap_id).await {
        Ok(Some(swap_request)) => {
            // Only the target user can respond to a targeted swap
            if swap_request.target_user_id.as_ref() != Some(&claims.sub) {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "Only the target user can respond to this swap",
                )));
            }

            // Can only respond to pending swaps
            if swap_request.status != ShiftSwapStatus::Pending {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "Can only respond to pending swaps",
                )));
            }

            match repo
                .respond_to_swap(swap_id, &claims.sub, true, response.notes.clone())
                .await
            {
                Ok(updated_swap) => Ok(HttpResponse::Ok().json(ApiResponse::success(updated_swap))),
                Err(err) => {
                    log::error!("Error responding to swap request: {}", err);
                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "Failed to respond to swap request",
                        )),
                    )
                }
            }
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Swap request not found")))
        }
        Err(err) => {
            log::error!("Error fetching swap request for response: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch swap request")))
        }
    }
}

/// Approve a shift swap request (managers/admins only)
pub async fn approve_swap_request(
    claims: Claims,
    repo: web::Data<ShiftSwapRepository>,
    path: web::Path<i64>,
    approval: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    // Only managers and admins can approve swap requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to approve swap requests",
        )));
    }

    let swap_id = path.into_inner();

    match repo
        .approve_swap(
            swap_id,
            &claims.sub,
            approval.notes.clone().unwrap_or_default(),
        )
        .await
    {
        Ok(approved_swap) => Ok(HttpResponse::Ok().json(ApiResponse::success(approved_swap))),
        Err(err) => {
            log::error!("Error approving swap request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to approve swap request")))
        }
    }
}

/// Deny a shift swap request (managers/admins only)
pub async fn deny_swap_request(
    claims: Claims,
    repo: web::Data<ShiftSwapRepository>,
    path: web::Path<i64>,
    denial: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    // Only managers and admins can deny swap requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to deny swap requests",
        )));
    }

    let swap_id = path.into_inner();

    match repo
        .deny_swap(
            swap_id,
            &claims.sub,
            denial.notes.clone().unwrap_or_default(),
        )
        .await
    {
        Ok(denied_swap) => Ok(HttpResponse::Ok().json(ApiResponse::success(denied_swap))),
        Err(err) => {
            log::error!("Error denying swap request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to deny swap request")))
        }
    }
}
