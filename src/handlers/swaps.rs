use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::database::models::{ShiftSwapInput, ShiftSwapStatus};
use crate::database::models::activity::Action;
use crate::database::repositories::shift_swap::ShiftSwapRepository;
use crate::database::repositories::company::CompanyRepository;
use crate::handlers::admin::ApiResponse;
use crate::services::auth::Claims;
use crate::services::activity_logger::ActivityLogger;

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
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    input: web::Json<ShiftSwapInput>,
    req: HttpRequest,
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
        request_input.requesting_user_id = claims.sub.clone();
    }

    let original_shift_id = request_input.original_shift_id;
    let target_user_id = request_input.target_user_id.clone();
    let requesting_user_id = request_input.requesting_user_id.clone();

    match repo.create_swap_request(request_input).await {
        Ok(swap_request) => {
            // Log shift swap request creation activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("original_shift_id".to_string(), serde_json::Value::Number(serde_json::Number::from(original_shift_id)));
                metadata.insert("requesting_user_id".to_string(), serde_json::Value::String(requesting_user_id.clone()));
                if let Some(target_id) = &target_user_id {
                    metadata.insert("target_user_id".to_string(), serde_json::Value::String(target_id.clone()));
                }
                
                if let Err(e) = activity_logger
                    .log_shift_swap_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        swap_request.id,
                        Action::CREATED,
                        format!("Shift swap request created by user {} for shift {}", requesting_user_id, original_shift_id),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log shift swap creation activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Created().json(ApiResponse::success(swap_request)))
        }
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
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<i64>,
    response: web::Json<SwapResponseRequest>,
    req: HttpRequest,
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

            let original_shift_id = swap_request.original_shift_id;
            let requesting_user_id = swap_request.requesting_user_id.clone();

            match repo
                .respond_to_swap(swap_id, &claims.sub, true, response.notes.clone())
                .await
            {
                Ok(updated_swap) => {
                    // Log swap response activity
                    if let Ok(Some(company)) = company_repo
                        .get_primary_company_for_user(&claims.sub)
                        .await
                    {
                        let mut metadata = HashMap::new();
                        metadata.insert("original_shift_id".to_string(), serde_json::Value::Number(serde_json::Number::from(original_shift_id)));
                        metadata.insert("requesting_user_id".to_string(), serde_json::Value::String(requesting_user_id.clone()));
                        metadata.insert("response_accepted".to_string(), serde_json::Value::Bool(true));
                        if let Some(target_shift_id) = response.target_shift_id {
                            metadata.insert("target_shift_id".to_string(), serde_json::Value::Number(serde_json::Number::from(target_shift_id)));
                        }
                        if let Some(notes) = &response.notes {
                            metadata.insert("response_notes".to_string(), serde_json::Value::String(notes.clone()));
                        }
                        
                        if let Err(e) = activity_logger
                            .log_shift_swap_activity(
                                company.id,
                                Some(claims.sub.parse().unwrap_or(0)),
                                swap_id,
                                Action::UPDATED,
                                format!("User {} responded to swap request from {}", claims.sub, requesting_user_id),
                                Some(metadata),
                                &req,
                            )
                            .await
                        {
                            log::warn!("Failed to log swap response activity: {}", e);
                        }
                    }
                    
                    Ok(HttpResponse::Ok().json(ApiResponse::success(updated_swap)))
                }
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
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<i64>,
    approval: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Only managers and admins can approve swap requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to approve swap requests",
        )));
    }

    let swap_id = path.into_inner();

    // Get swap details before approval for logging
    let swap_request = match repo.get_swap_by_id(swap_id).await {
        Ok(Some(swap)) => swap,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Swap request not found")));
        }
        Err(err) => {
            log::error!("Error fetching swap request for approval: {}", err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch swap request")));
        }
    };

    match repo
        .approve_swap(
            swap_id,
            &claims.sub,
            approval.notes.clone().unwrap_or_default(),
        )
        .await
    {
        Ok(approved_swap) => {
            // Log swap approval activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("original_shift_id".to_string(), serde_json::Value::Number(serde_json::Number::from(swap_request.original_shift_id)));
                metadata.insert("requesting_user_id".to_string(), serde_json::Value::String(swap_request.requesting_user_id.clone()));
                if let Some(target_user_id) = &swap_request.target_user_id {
                    metadata.insert("target_user_id".to_string(), serde_json::Value::String(target_user_id.clone()));
                }
                if let Some(notes) = &approval.notes {
                    metadata.insert("approval_notes".to_string(), serde_json::Value::String(notes.clone()));
                }
                
                if let Err(e) = activity_logger
                    .log_shift_swap_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        swap_id,
                        Action::APPROVED,
                        format!("Shift swap request approved for user {}", swap_request.requesting_user_id),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log swap approval activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(approved_swap)))
        }
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
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<i64>,
    denial: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Only managers and admins can deny swap requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to deny swap requests",
        )));
    }

    let swap_id = path.into_inner();

    // Get swap details before denial for logging
    let swap_request = match repo.get_swap_by_id(swap_id).await {
        Ok(Some(swap)) => swap,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Swap request not found")));
        }
        Err(err) => {
            log::error!("Error fetching swap request for denial: {}", err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch swap request")));
        }
    };

    match repo
        .deny_swap(
            swap_id,
            &claims.sub,
            denial.notes.clone().unwrap_or_default(),
        )
        .await
    {
        Ok(denied_swap) => {
            // Log swap denial activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("original_shift_id".to_string(), serde_json::Value::Number(serde_json::Number::from(swap_request.original_shift_id)));
                metadata.insert("requesting_user_id".to_string(), serde_json::Value::String(swap_request.requesting_user_id.clone()));
                if let Some(target_user_id) = &swap_request.target_user_id {
                    metadata.insert("target_user_id".to_string(), serde_json::Value::String(target_user_id.clone()));
                }
                if let Some(notes) = &denial.notes {
                    metadata.insert("denial_notes".to_string(), serde_json::Value::String(notes.clone()));
                }
                
                if let Err(e) = activity_logger
                    .log_shift_swap_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        swap_id,
                        Action::REJECTED,
                        format!("Shift swap request denied for user {}", swap_request.requesting_user_id),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log swap denial activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(denied_swap)))
        }
        Err(err) => {
            log::error!("Error denying swap request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to deny swap request")))
        }
    }
}
