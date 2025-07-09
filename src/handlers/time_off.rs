use actix_web::{web, HttpResponse, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::auth::Claims;
use crate::database::models::{TimeOffRequestInput, TimeOffStatus};
use crate::database::time_off_repository::TimeOffRepository;
use crate::handlers::admin::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct TimeOffQuery {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}

/// Create a new time-off request
pub async fn create_time_off_request(
    claims: Claims,
    repo: web::Data<TimeOffRepository>,
    input: web::Json<TimeOffRequestInput>,
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
        request_input.user_id = claims.sub;
    }

    match repo.create_request(request_input).await {
        Ok(request) => Ok(HttpResponse::Created().json(ApiResponse::success(request))),
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
    path: web::Path<i64>,
    input: web::Json<TimeOffRequestInput>,
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

            // Ensure user_id doesn't change for non-admin/manager users
            if !claims.is_admin() && !claims.is_manager() {
                request_input.user_id = existing_request.user_id;
            }

            match repo.update_request(request_id, request_input).await {
                Ok(updated_request) => {
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
    path: web::Path<i64>,
    approval: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    // Only managers and admins can approve requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to approve requests",
        )));
    }

    let request_id = path.into_inner();

    match repo
        .approve_request(request_id, &claims.sub, approval.notes.clone())
        .await
    {
        Ok(approved_request) => Ok(HttpResponse::Ok().json(ApiResponse::success(approved_request))),
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
    path: web::Path<i64>,
    denial: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    // Only managers and admins can deny requests
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to deny requests",
        )));
    }

    let request_id = path.into_inner();

    match repo
        .deny_request(request_id, &claims.sub, denial.notes.clone())
        .await
    {
        Ok(denied_request) => Ok(HttpResponse::Ok().json(ApiResponse::success(denied_request))),
        Err(err) => {
            log::error!("Error denying time-off request: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to deny time-off request")))
        }
    }
}
