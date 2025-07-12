use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::NaiveDateTime;
use serde::Deserialize;
use std::collections::HashMap;

use crate::database::models::{PtoBalanceType, TimeOffRequestInput, TimeOffStatus, TimeOffType, Action};
use crate::database::repositories::pto_balance::PtoBalanceRepository;
use crate::database::repositories::time_off::TimeOffRepository;
use crate::database::repositories::company::CompanyRepository;
use crate::handlers::admin::ApiResponse;
use crate::services::auth::Claims;
use crate::services::ActivityLogger;

#[derive(Debug, Deserialize)]
pub struct TimeOffQuery {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}

/// Create a new time-off request
pub async fn create_time_off_request(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    input: web::Json<TimeOffRequestInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Users can only create requests for themselves unless they're managers/admins
    let mut request_input = input.into_inner();

    if !claims.is_admin() && !claims.is_manager() && request_input.user_id != claims.sub {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Can only create requests for yourself",
        )));
    }

    // If employee, force user_id to be their own ID
    if !claims.is_admin() && !claims.is_manager() {
        request_input.user_id = claims.sub.clone();
    }

    let request_type = request_input.request_type.clone();
    let start_date = request_input.start_date;
    let end_date = request_input.end_date;
    let requesting_user_id = request_input.user_id.clone();

    match repo.create_request(request_input).await {
        Ok(request) => {
            // Log time-off request creation activity
            // Get user's primary company for logging
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("request_type".to_string(), serde_json::Value::String(format!("{:?}", request_type)));
                metadata.insert("start_date".to_string(), serde_json::Value::String(start_date.to_string()));
                metadata.insert("end_date".to_string(), serde_json::Value::String(end_date.to_string()));
                metadata.insert("requesting_user".to_string(), serde_json::Value::String(requesting_user_id.clone()));
                
                if let Err(e) = activity_logger
                    .log_time_off_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        request.id,
                        Action::CREATED,
                        format!("Time-off request created for user {}", requesting_user_id),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log time-off request creation activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Created().json(ApiResponse::success(request)))
        }
        Err(err) => {
            log::error!("Error creating time-off request: {}", err);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create time-off request",
                )),
            )
        }
    }
}

/// Get time-off requests with optional filtering
pub async fn get_time_off_requests(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    query: web::Query<TimeOffQuery>,
) -> Result<HttpResponse> {
    // Employees can only see their own requests
    let user_id = if !claims.is_admin() && !claims.is_manager() {
        Some(claims.sub.as_str())
    } else {
        query.user_id.as_deref()
    };

    // Convert status string to enum if provided
    let status_filter = if let Some(status_str) = &query.status {
        match status_str.parse::<TimeOffStatus>() {
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

    match repo.get_requests(user_id, status_filter, None, None).await {
        Ok(requests) => Ok(HttpResponse::Ok().json(ApiResponse::success(requests))),
        Err(err) => {
            log::error!("Error fetching time-off requests: {}", err);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to fetch time-off requests",
                )),
            )
        }
    }
}

/// Get a specific time-off request by ID
pub async fn get_time_off_request(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let request_id = path.into_inner();

    match repo.get_request_by_id(request_id).await {
        Ok(Some(request)) => {
            // Check if user has permission to view this request
            if !claims.is_admin() && !claims.is_manager() && request.user_id != claims.sub {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "Cannot view other users' requests",
                )));
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(request)))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Time-off request not found")))
        }
        Err(err) => {
            log::error!("Error fetching time-off request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch time-off request")))
        }
    }
}

/// Update a time-off request
pub async fn update_time_off_request(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<i64>,
    input: web::Json<TimeOffRequestInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let request_id = path.into_inner();

    // First check if the request exists and get current state
    match repo.get_request_by_id(request_id).await {
        Ok(Some(existing_request)) => {
            // Check permissions - users can only update their own pending requests
            if !claims.is_admin() && !claims.is_manager() && existing_request.user_id != claims.sub
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "Cannot update other users' requests",
                )));
            }

            // Only allow updates to pending requests
            if existing_request.status != TimeOffStatus::Pending {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "Cannot update non-pending requests",
                )));
            }

            let mut request_input = input.into_inner();
            let new_request_type = request_input.request_type.clone();
            let new_start_date = request_input.start_date;
            let new_end_date = request_input.end_date;

            // Ensure user_id doesn't change for non-admin/manager users
            if !claims.is_admin() && !claims.is_manager() {
                request_input.user_id = existing_request.user_id.clone();
            }

            match repo.update_request(request_id, request_input).await {
                Ok(updated_request) => {
                    // Log time-off request update activity
                    if let Ok(Some(company)) = company_repo
                        .get_primary_company_for_user(&claims.sub)
                        .await
                    {
                        let mut metadata = HashMap::new();
                        metadata.insert("request_type".to_string(), serde_json::Value::String(format!("{:?}", new_request_type)));
                        metadata.insert("start_date".to_string(), serde_json::Value::String(new_start_date.to_string()));
                        metadata.insert("end_date".to_string(), serde_json::Value::String(new_end_date.to_string()));
                        metadata.insert("target_user".to_string(), serde_json::Value::String(updated_request.user_id.clone()));
                        
                        // Add previous values for comparison
                        metadata.insert("previous_request_type".to_string(), serde_json::Value::String(format!("{:?}", existing_request.request_type)));
                        metadata.insert("previous_start_date".to_string(), serde_json::Value::String(existing_request.start_date.to_string()));
                        metadata.insert("previous_end_date".to_string(), serde_json::Value::String(existing_request.end_date.to_string()));
                        
                        if let Err(e) = activity_logger
                            .log_time_off_activity(
                                company.id,
                                Some(claims.sub.parse().unwrap_or(0)),
                                request_id,
                                Action::UPDATED,
                                format!("Time-off request updated for user {}", updated_request.user_id),
                                Some(metadata),
                                &req,
                            )
                            .await
                        {
                            log::warn!("Failed to log time-off request update activity: {}", e);
                        }
                    }
                    
                    Ok(HttpResponse::Ok().json(ApiResponse::success(updated_request)))
                }
                Err(err) => {
                    log::error!("Error updating time-off request: {}", err);
                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "Failed to update time-off request",
                        )),
                    )
                }
            }
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Time-off request not found")))
        }
        Err(err) => {
            log::error!("Error fetching time-off request for update: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch time-off request")))
        }
    }
}

/// Delete a time-off request
pub async fn delete_time_off_request(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let request_id = path.into_inner();

    // First check if the request exists and get current state
    match repo.get_request_by_id(request_id).await {
        Ok(Some(existing_request)) => {
            // Check permissions - users can only delete their own pending requests
            if !claims.is_admin() && !claims.is_manager() && existing_request.user_id != claims.sub
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "Cannot delete other users' requests",
                )));
            }

            // Only allow deletion of pending requests
            if existing_request.status != TimeOffStatus::Pending {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "Cannot delete non-pending requests",
                )));
            }

            match repo.delete_request(request_id).await {
                Ok(_) => Ok(HttpResponse::NoContent().finish()),
                Err(err) => {
                    log::error!("Error deleting time-off request: {}", err);
                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "Failed to delete time-off request",
                        )),
                    )
                }
            }
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Time-off request not found")))
        }
        Err(err) => {
            log::error!("Error fetching time-off request for deletion: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch time-off request")))
        }
    }
}

/// Approve a time-off request (managers/admins only)
pub async fn approve_time_off_request(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    pto_repo: web::Data<PtoBalanceRepository>,
    path: web::Path<i64>,
    approval: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Only managers and admins can approve requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to approve requests",
        )));
    }

    let request_id = path.into_inner();

    // First, get the time-off request to check balance requirements
    let time_off_request = match repo.get_request_by_id(request_id).await {
        Ok(Some(request)) => request,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Time-off request not found")));
        }
        Err(err) => {
            log::error!("Error fetching time-off request: {}", err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch time-off request")));
        }
    };

    // Calculate hours for the request (simple calculation: 8 hours per day)
    let days = (time_off_request.end_date - time_off_request.start_date).num_days() + 1;
    let hours_needed = (days * 8) as i32;

    // Map TimeOffType to PtoBalanceType
    let balance_type = match time_off_request.request_type {
        TimeOffType::Vacation => PtoBalanceType::Pto,
        TimeOffType::Sick => PtoBalanceType::Sick,
        TimeOffType::Personal => PtoBalanceType::Personal,
        TimeOffType::Emergency => PtoBalanceType::Personal,
        TimeOffType::Bereavement => PtoBalanceType::Personal,
        TimeOffType::MaternityPaternity => PtoBalanceType::Pto,
        TimeOffType::Other => PtoBalanceType::Pto, // Default to PTO for other types
    };

    // Check if user has sufficient balance
    let user_balance = match pto_repo.get_balance(&time_off_request.user_id).await {
        Ok(Some(balance)) => balance,
        Ok(None) => {
            return Ok(
                HttpResponse::BadRequest().json(ApiResponse::<()>::error("User balance not found"))
            );
        }
        Err(err) => {
            log::error!("Error fetching user balance: {}", err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch user balance")));
        }
    };

    let available_balance = match balance_type {
        PtoBalanceType::Pto => user_balance.pto_balance_hours,
        PtoBalanceType::Sick => user_balance.sick_balance_hours,
        PtoBalanceType::Personal => user_balance.personal_balance_hours,
    };

    if available_balance < hours_needed {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error(&format!(
                "Insufficient balance: {} hours needed, {} available",
                hours_needed, available_balance
            ))),
        );
    }

    // Approve the request
    let balance_type_for_logging = balance_type.clone();
    match repo
        .approve_request(request_id, &claims.sub, approval.notes.clone())
        .await
    {
        Ok(approved_request) => {
            // Deduct PTO balance
            match pto_repo
                .use_balance_for_time_off(
                    &time_off_request.user_id,
                    request_id,
                    balance_type,
                    hours_needed,
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "PTO balance deducted for user {} (request {}): {} hours",
                        time_off_request.user_id,
                        request_id,
                        hours_needed
                    );
                    
                    // Log time-off request approval activity
                    if let Ok(Some(company)) = company_repo
                        .get_primary_company_for_user(&claims.sub)
                        .await
                    {
                        let mut metadata = HashMap::new();
                        metadata.insert("request_type".to_string(), serde_json::Value::String(format!("{:?}", time_off_request.request_type)));
                        metadata.insert("target_user".to_string(), serde_json::Value::String(time_off_request.user_id.clone()));
                        metadata.insert("hours_deducted".to_string(), serde_json::Value::Number(serde_json::Number::from(hours_needed)));
                        metadata.insert("balance_type".to_string(), serde_json::Value::String(format!("{:?}", balance_type_for_logging)));
                        if let Some(notes) = &approval.notes {
                            metadata.insert("approval_notes".to_string(), serde_json::Value::String(notes.clone()));
                        }
                        
                        if let Err(e) = activity_logger
                            .log_time_off_activity(
                                company.id,
                                Some(claims.sub.parse().unwrap_or(0)),
                                request_id,
                                Action::APPROVED,
                                format!("Time-off request approved for user {}", time_off_request.user_id),
                                Some(metadata),
                                &req,
                            )
                            .await
                        {
                            log::warn!("Failed to log time-off request approval activity: {}", e);
                        }
                    }
                    
                    Ok(HttpResponse::Ok().json(ApiResponse::success(approved_request)))
                }
                Err(err) => {
                    log::error!("Error deducting PTO balance: {}", err);
                    // TODO: Consider rolling back the approval if balance deduction fails
                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "Request approved but balance deduction failed",
                        )),
                    )
                }
            }
        }
        Err(err) => {
            log::error!("Error approving time-off request: {}", err);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to approve time-off request",
                )),
            )
        }
    }
}

/// Deny a time-off request (managers/admins only)
pub async fn deny_time_off_request(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<i64>,
    denial: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Only managers and admins can deny requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to deny requests",
        )));
    }

    let request_id = path.into_inner();

    // Get the time-off request details for logging
    let time_off_request = match repo.get_request_by_id(request_id).await {
        Ok(Some(request)) => request,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Time-off request not found")));
        }
        Err(err) => {
            log::error!("Error fetching time-off request: {}", err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch time-off request")));
        }
    };

    match repo
        .deny_request(request_id, &claims.sub, denial.notes.clone())
        .await
    {
        Ok(denied_request) => {
            // Log time-off request denial activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("request_type".to_string(), serde_json::Value::String(format!("{:?}", time_off_request.request_type)));
                metadata.insert("target_user".to_string(), serde_json::Value::String(time_off_request.user_id.clone()));
                if let Some(notes) = &denial.notes {
                    metadata.insert("denial_notes".to_string(), serde_json::Value::String(notes.clone()));
                }
                
                if let Err(e) = activity_logger
                    .log_time_off_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        request_id,
                        Action::REJECTED,
                        format!("Time-off request denied for user {}", time_off_request.user_id),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log time-off request denial activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(denied_request)))
        }
        Err(err) => {
            log::error!("Error denying time-off request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to deny time-off request")))
        }
    }
}

/// Wrapper function for approving time-off requests with PTO balance integration
async fn approve_time_off_with_balance_check(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    pto_repo: web::Data<PtoBalanceRepository>,
    path: web::Path<i64>,
    approval: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    approve_time_off_request(claims, repo, company_repo, activity_logger, pto_repo, path, approval, req).await
}

/// Public wrapper for the approve endpoint
pub async fn approve_time_off_request_endpoint(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    pto_repo: web::Data<PtoBalanceRepository>,
    path: web::Path<i64>,
    approval: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    approve_time_off_with_balance_check(claims, repo, company_repo, activity_logger, pto_repo, path, approval, req).await
}
