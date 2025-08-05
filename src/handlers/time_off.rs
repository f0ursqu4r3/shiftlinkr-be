use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::database::models::{
    Action, PtoBalanceType, TimeOffRequestInput, TimeOffStatus, TimeOffType,
};
use crate::database::repositories::{pto_balance as pto_repo, time_off as time_off_repo};
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::{activity_logger, user_context::extract_context};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeOffQuery {
    pub user_id: Option<Uuid>,
    pub status: Option<TimeOffStatus>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}

/// Create a new time-off request
pub async fn create_time_off_request(
    input: web::Json<TimeOffRequestInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let request_input = input.into_inner();
    let request_user_id = request_input.user_id.clone();

    user_context.requires_same_user(request_user_id)?;

    let request_type = request_input.request_type.clone();
    let start_date = request_input.start_date;
    let end_date = request_input.end_date;
    let requesting_user_id = request_input.user_id.clone();
    let company_id = request_input.company_id.clone();

    user_context.requires_same_company(company_id)?;

    let time_off_request = time_off_repo::create_request(request_input)
        .await
        .map_err(|e| {
            log::error!("Failed to create time-off request: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Log the time-off request creation activity
    let metadata = activity_logger::metadata(vec![
        (&"request_type", request_type.to_string()),
        (&"start_date", start_date.to_string()),
        (&"end_date", end_date.to_string()),
        (&"requesting_user", requesting_user_id.to_string()),
    ]);

    if let Err(e) = activity_logger::log_time_off_activity(
        company_id,
        Some(user_context.user_id()),
        time_off_request.id,
        Action::CREATED,
        format!("Time-off request created for user {}", requesting_user_id),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log time-off request creation activity: {}", e);
    }

    Ok(ApiResponse::created(time_off_request))
}

/// Get time-off requests with optional filtering
pub async fn get_time_off_requests(
    query: web::Query<TimeOffQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let target_user_id = query.user_id.unwrap_or(user_context.user_id());
    let status_filter = query.status.clone();
    let start_date = query.start_date;
    let end_date = query.end_date;

    user_context.requires_same_user_or(target_user_id, "Cannot view other users' requests")?;

    let time_off_requests =
        time_off_repo::get_requests(Some(target_user_id), status_filter, start_date, end_date)
            .await
            .map_err(|e| {
                log::error!("Error fetching time-off requests: {}", e);
                AppError::DatabaseError(e)
            })?;

    Ok(ApiResponse::success(time_off_requests))
}

/// Get a specific time-off request by ID
pub async fn get_time_off_request(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let request_id = path.into_inner();

    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching time-off request: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Time-off request not found: {}", request_id);
            AppError::NotFound(format!("Time-off request not found: {}", request_id))
        })?;

    user_context.requires_same_user_or(
        time_off_request.user_id,
        "Cannot view other users' requests",
    )?;

    Ok(ApiResponse::success(time_off_request))
}

/// Update a time-off request
pub async fn update_time_off_request(
    path: web::Path<Uuid>,
    input: web::Json<TimeOffRequestInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let request_id = path.into_inner();

    let request_input = input.into_inner();
    let requesting_user_id = request_input.user_id.clone();
    let new_request_type = request_input.request_type.clone();
    let new_start_date = request_input.start_date;
    let new_end_date = request_input.end_date;

    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching time-off request for update: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Time-off request not found: {}", request_id);
            AppError::NotFound(format!("Time-off request not found: {}", request_id))
        })?;

    user_context.requires_same_user_or(
        time_off_request.user_id,
        "Cannot update other users' requests",
    )?;

    // Only allow updates to pending requests
    if time_off_request.status != TimeOffStatus::Pending {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Cannot update non-pending requests",
        )));
    }

    if time_off_request.user_id != requesting_user_id {
        return Ok(HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("Cannot update request user ID")));
    }

    let updated_request = time_off_repo::update_request(request_id, request_input)
        .await
        .map_err(|e| {
            log::error!("Error fetching time-off request for update: {}", e);
            AppError::DatabaseError(e)
        })?;

    let company_id = user_context.strict_company_id()?;

    let metadata = activity_logger::metadata(vec![
        (&"request_type", new_request_type.to_string()),
        (&"start_date", new_start_date.to_string()),
        (&"end_date", new_end_date.to_string()),
        (&"target_user", updated_request.user_id.to_string()),
        (
            &"previous_request_type",
            format!("{:?}", time_off_request.request_type),
        ),
        (
            &"previous_start_date",
            time_off_request.start_date.to_string(),
        ),
        (&"previous_end_date", time_off_request.end_date.to_string()),
    ]);

    if let Err(e) = activity_logger::log_time_off_activity(
        company_id,
        Some(user_context.user_id()),
        request_id, // Use request_id directly
        Action::UPDATED,
        format!(
            "Time-off request updated for user {}",
            updated_request.user_id
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log time-off request update activity: {}", e);
    }

    Ok(ApiResponse::success(updated_request))
}

/// Delete a time-off request
pub async fn delete_time_off_request(
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let request_id = path.into_inner();

    // First check if the request exists and get current state
    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching time-off request for deletion: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Time-off request not found for deletion: {}", request_id);
            AppError::NotFound(format!("Time-off request not found: {}", request_id))
        })?;

    let target_user_id = time_off_request.user_id;

    user_context.requires_same_user_or(target_user_id, "Cannot delete other users' requests")?;

    // Only allow deletion of pending requests
    if time_off_request.status != TimeOffStatus::Pending {
        return Err(AppError::BadRequest("Cannot delete non-pending requests".to_string()).into());
    }

    time_off_repo::delete_request(request_id)
        .await
        .map_err(|e| {
            log::error!("Error deleting time-off request: {}", e);
            AppError::DatabaseError(e)
        })?;

    let company_id = user_context.strict_company_id()?;

    let metadata = activity_logger::metadata(vec![("target_user", target_user_id.to_string())]);

    if let Err(e) = activity_logger::log_time_off_activity(
        company_id,
        Some(user_context.user_id()),
        request_id,
        Action::DELETED,
        format!("Time-off request deleted for user {}", target_user_id),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log time-off request deletion activity: {}", e);
    }

    Ok(ApiResponse::success_message(
        "Time-off request deleted successfully",
    ))
}

/// Approve a time-off request (managers/admins only)
pub async fn approve_time_off_request(
    path: web::Path<Uuid>,
    approval: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let request_id = path.into_inner();
    let company_id = user_context.strict_company_id()?;

    // First, get the time-off request to check balance requirements
    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching time-off request for approval: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Time-off request not found for approval: {}", request_id);
            AppError::NotFound(format!("Time-off request not found: {}", request_id))
        })?;

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
    let user_balance = pto_repo::get_balance_for_company(time_off_request.user_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching PTO balance: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!(
                "PTO balance not found for user: {}",
                time_off_request.user_id
            );
            AppError::NotFound(format!(
                "PTO balance not found for user: {}",
                time_off_request.user_id
            ))
        })?;

    let available_balance = match balance_type {
        PtoBalanceType::Pto => user_balance.pto_balance_hours,
        PtoBalanceType::Sick => user_balance.sick_balance_hours,
        PtoBalanceType::Personal => user_balance.personal_balance_hours,
    };

    if available_balance < hours_needed {
        return Err(AppError::BadRequest(format!(
            "Insufficient {} balance: requested {} hours, available {} hours",
            balance_type, hours_needed, available_balance
        ))
        .into());
    }

    // Approve the request
    let balance_type_for_logging = balance_type.clone();
    let approved_request =
        time_off_repo::approve_request(request_id, user_context.user.id, approval.notes.clone())
            .await
            .map_err(|e| {
                log::error!("Error approving time-off request: {}", e);
                AppError::DatabaseError(e)
            })?;

    // Deduct PTO balance
    pto_repo::use_balance_for_time_off_for_company(
        time_off_request.user_id,
        company_id,
        request_id,
        balance_type,
        hours_needed,
    )
    .await
    .map_err(|e| {
        log::error!("Error deducting PTO balance: {}", e);
        AppError::DatabaseError(e)
    })?;

    // Log time-off request approval activity
    let metadata = activity_logger::metadata(vec![
        ("request_type", time_off_request.request_type.to_string()),
        ("target_user", time_off_request.user_id.to_string()),
        ("hours_deducted", hours_needed.to_string()),
        ("balance_type", format!("{:?}", balance_type_for_logging)),
        ("approval_notes", approval.notes.clone().unwrap_or_default()),
    ]);

    if let Err(e) = activity_logger::log_time_off_activity(
        company_id,
        Some(user_context.user.id),
        request_id,
        Action::APPROVED,
        format!(
            "Time-off request approved for user {}",
            time_off_request.user_id
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log time-off request approval activity: {}", e);
    }

    Ok(ApiResponse::success(approved_request))
}

/// Deny a time-off request (managers/admins only)
pub async fn deny_time_off_request(
    path: web::Path<Uuid>,
    denial: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    // Only managers and admins can deny requests
    user_context.requires_manager()?;

    let request_id = path.into_inner();
    let company_id = user_context.strict_company_id()?;

    // Get the time-off request details for logging
    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching time-off request for denial: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Time-off request not found for denial: {}", request_id);
            AppError::NotFound(format!("Time-off request not found: {}", request_id))
        })?;

    let denied_request =
        time_off_repo::deny_request(request_id, user_context.user.id, denial.notes.clone())
            .await
            .map_err(|e| {
                log::error!("Error denying time-off request: {}", e);
                AppError::DatabaseError(e)
            })?;

    // Log time-off request denial activity
    let metadata = activity_logger::metadata(vec![
        ("request_type", time_off_request.request_type.to_string()),
        ("target_user", time_off_request.user_id.to_string()),
        ("denial_notes", denial.notes.clone().unwrap_or_default()),
    ]);

    if let Err(e) = activity_logger::log_time_off_activity(
        company_id,
        Some(user_context.user.id),
        request_id,
        Action::REJECTED,
        format!(
            "Time-off request denied for user {}",
            time_off_request.user_id
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log time-off request denial activity: {}", e);
    }

    Ok(ApiResponse::success(denied_request))
}

// /// Wrapper function for approving time-off requests with PTO balance integration
// async fn approve_time_off_with_balance_check(
//     AsyncUserContext(user_context): AsyncUserContext,
//     time_off_repo: web::Data<TimeOffRepository>,
//     activity_logger: web::Data<ActivityLogger>,
//     pto_repo: web::Data<PtoBalanceRepository>,
//     path: web::Path<Uuid>,
//     approval: web::Json<ApprovalRequest>,
//     req: HttpRequest,
// ) -> Result<HttpResponse> {
//     approve_time_off_request(
//         user_context,
//         time_off_repo,
//         activity_logger,
//         pto_repo,
//         path,
//         approval,
//         req,
//     )
//     .await
// }

// /// Public wrapper for the approve endpoint
// pub async fn approve_time_off_request_endpoint(
//     AsyncUserContext(user_context): AsyncUserContext,
//     time_off_repo: web::Data<TimeOffRepository>,
//     activity_logger: web::Data<ActivityLogger>,
//     pto_repo: web::Data<PtoBalanceRepository>,
//     path: web::Path<Uuid>,
//     approval: web::Json<ApprovalRequest>,
//     req: HttpRequest,
// ) -> Result<HttpResponse> {
//     approve_time_off_with_balance_check(
//         user_context,
//         time_off_repo,
//         activity_logger,
//         pto_repo,
//         path,
//         approval,
//         req,
//     )
//     .await
// }
