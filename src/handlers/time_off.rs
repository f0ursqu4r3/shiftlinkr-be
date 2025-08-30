use actix_web::{HttpResponse, Result, web};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        models::{Action, PtoBalanceType, TimeOffRequestInput, TimeOffStatus, TimeOffType},
        repositories::{pto_balance as pto_repo, time_off as time_off_repo},
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::{cache::InvalidationContext, request_info::RequestInfo},
    services::{activity_logger, user_context::UserContext},
};

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
    ctx: UserContext,
    input: web::Json<TimeOffRequestInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let request_input = input.into_inner();
    let request_user_id = request_input.user_id.clone();

    ctx.requires_same_user(request_user_id)?;

    let request_type = request_input.request_type.clone();
    let start_date = request_input.start_date;
    let end_date = request_input.end_date;
    let requesting_user_id = request_input.user_id.clone();
    let company_id = request_input.company_id.clone();

    ctx.requires_same_company(company_id)?;

    let time_off_request = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let time_off_request = time_off_repo::create_request(tx, request_input).await?;

            // Log the time-off request creation activity
            let metadata = activity_logger::metadata(vec![
                ("request_type", request_type.to_string()),
                ("start_date", start_date.to_string()),
                ("end_date", end_date.to_string()),
                ("requesting_user", requesting_user_id.to_string()),
            ]);

            activity_logger::log_time_off_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                time_off_request.id,
                &Action::CREATED,
                "Time-off request created".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(time_off_request)
        })
    })
    .await?;

    // Smart cache invalidation - create_time_off_request
    cache
        .invalidate(
            "time-off",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(requesting_user_id),
                ..Default::default()
            },
        )
        .await;

    // Time-off affects shift availability and stats
    cache
        .invalidate(
            "shifts",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::created(time_off_request))
}

/// Get time-off requests with optional filtering
pub async fn get_time_off_requests(
    query: web::Query<TimeOffQuery>,
    ctx: UserContext,
) -> Result<HttpResponse> {
    let target_user_id = query.user_id.unwrap_or(ctx.user_id());
    let status_filter = query.status.clone();
    let start_date = query.start_date;
    let end_date = query.end_date;

    ctx.requires_same_user_or(target_user_id, "Cannot view other users' requests")?;

    let time_off_requests =
        time_off_repo::get_requests(Some(target_user_id), status_filter, start_date, end_date)
            .await
            .map_err(AppError::from)?;

    Ok(ApiResponse::success(time_off_requests))
}

/// Get a specific time-off request by ID
pub async fn get_time_off_request(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let request_id = path.into_inner();

    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Time-off request not found: {}", request_id)))?;

    ctx.requires_same_user_or(
        time_off_request.user_id,
        "Cannot view other users' requests",
    )?;

    Ok(ApiResponse::success(time_off_request))
}

/// Update a time-off request
pub async fn update_time_off_request(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<TimeOffRequestInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let request_id = path.into_inner();

    let request_input = input.into_inner();
    let requesting_user_id = request_input.user_id.clone();
    let new_request_type = request_input.request_type.clone();
    let new_start_date = request_input.start_date;
    let new_end_date = request_input.end_date;
    let company_id = ctx.strict_company_id()?;

    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Time-off request not found: {}", request_id)))?;

    ctx.requires_same_user_or(
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

    let updated_request = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let updated_request =
                time_off_repo::update_request(tx, request_id, request_input).await?;

            let company_id = ctx.strict_company_id()?;

            let metadata = activity_logger::metadata(vec![
                ("request_type", new_request_type.to_string()),
                ("start_date", new_start_date.to_string()),
                ("end_date", new_end_date.to_string()),
                ("target_user", updated_request.user_id.to_string()),
                (
                    "previous_request_type",
                    format!("{:?}", time_off_request.request_type),
                ),
                (
                    "previous_start_date",
                    time_off_request.start_date.to_string(),
                ),
                ("previous_end_date", time_off_request.end_date.to_string()),
            ]);

            activity_logger::log_time_off_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                request_id,
                &Action::UPDATED,
                "Time-off request updated".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(updated_request)
        })
    })
    .await?;

    // Smart cache invalidation - update_time_off_request
    cache
        .invalidate(
            "time-off",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(requesting_user_id),
                resource_id: Some(request_id),
                ..Default::default()
            },
        )
        .await;

    // Time-off affects shift availability and stats
    cache
        .invalidate(
            "shifts",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success(updated_request))
}

/// Delete a time-off request
pub async fn delete_time_off_request(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let request_id = path.into_inner();

    // First check if the request exists and get current state
    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Time-off request not found: {}", request_id)))?;

    let target_user_id = time_off_request.user_id;
    let company_id = ctx.strict_company_id()?;

    ctx.requires_same_user_or(target_user_id, "Cannot delete other users' requests")?;

    // Only allow deletion of pending requests
    if time_off_request.status != TimeOffStatus::Pending {
        return Err(AppError::BadRequest("Cannot delete non-pending requests".to_string()).into());
    }

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            time_off_repo::delete_request(tx, request_id).await?;

            let company_id = ctx.strict_company_id()?;

            let metadata =
                activity_logger::metadata(vec![("target_user", target_user_id.to_string())]);

            activity_logger::log_time_off_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                request_id,
                &Action::DELETED,
                "Time-off request deleted".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    // Smart cache invalidation - delete_time_off_request
    cache
        .invalidate(
            "time-off",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(target_user_id),
                resource_id: Some(request_id),
                ..Default::default()
            },
        )
        .await;

    // Time-off affects shift availability and stats
    cache
        .invalidate(
            "shifts",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success_message(
        "Time-off request deleted successfully",
    ))
}

/// Approve a time-off request (managers/admins only)
pub async fn approve_time_off_request(
    path: web::Path<Uuid>,
    ctx: UserContext,
    approval: web::Json<ApprovalRequest>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let request_id = path.into_inner();
    let company_id = ctx.strict_company_id()?;

    // First, get the time-off request to check balance requirements
    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Time-off request not found: {}", request_id)))?;

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
        .map_err(AppError::from)?
        .ok_or_else(|| {
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

    let approved_request = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Approve the request
            let balance_type_for_logging = balance_type.clone();
            let approved_request = time_off_repo::approve_request(
                tx,
                request_id,
                ctx.user_id(),
                approval.notes.clone(),
            )
            .await?;

            // Deduct PTO balance
            pto_repo::use_balance_for_time_off_for_company(
                tx,
                time_off_request.user_id,
                company_id,
                request_id,
                balance_type,
                hours_needed,
            )
            .await?;

            // Log time-off request approval activity
            let metadata = activity_logger::metadata(vec![
                ("request_type", time_off_request.request_type.to_string()),
                ("target_user", time_off_request.user_id.to_string()),
                ("hours_deducted", hours_needed.to_string()),
                ("balance_type", format!("{:?}", balance_type_for_logging)),
                ("approval_notes", approval.notes.clone().unwrap_or_default()),
            ]);

            activity_logger::log_time_off_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                request_id,
                &Action::APPROVED,
                "Time-off request approved".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(approved_request)
        })
    })
    .await?;

    // Smart cache invalidation - approve_time_off_request
    cache
        .invalidate(
            "time-off",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(approved_request.user_id),
                resource_id: Some(request_id),
                ..Default::default()
            },
        )
        .await;

    // Time-off affects shift availability and stats
    cache
        .invalidate(
            "shifts",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success(approved_request))
}

/// Deny a time-off request (managers/admins only)
pub async fn deny_time_off_request(
    path: web::Path<Uuid>,
    ctx: UserContext,
    denial: web::Json<ApprovalRequest>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    // Only managers and admins can deny requests
    ctx.requires_manager()?;

    let request_id = path.into_inner();
    let company_id = ctx.strict_company_id()?;

    // Get the time-off request details for logging
    let time_off_request = time_off_repo::get_request_by_id(request_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Time-off request not found: {}", request_id)))?;

    let denied_request = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let denied_request =
                time_off_repo::deny_request(tx, request_id, ctx.user_id(), denial.notes.clone())
                    .await?;

            // Log time-off request denial activity
            let metadata = activity_logger::metadata(vec![
                ("request_type", time_off_request.request_type.to_string()),
                ("target_user", time_off_request.user_id.to_string()),
                ("denial_notes", denial.notes.clone().unwrap_or_default()),
            ]);

            activity_logger::log_time_off_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                request_id,
                &Action::REJECTED,
                "Time-off request denied".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(denied_request)
        })
    })
    .await?;

    // Smart cache invalidation - deny_time_off_request
    cache
        .invalidate(
            "time-off",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(denied_request.user_id),
                resource_id: Some(request_id),
                ..Default::default()
            },
        )
        .await;

    // Time-off affects shift availability and stats
    cache
        .invalidate(
            "shifts",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

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
