use actix_web::{
    HttpResponse, Result,
    web::{Data, Json, Path},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        models::{ShiftAssignmentInput, UserShiftScheduleInput},
        repositories::schedule as schedule_repo,
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::{CacheLayer, cache::InvalidationContext, request_info::RequestInfo},
    services::user_context::UserContext,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentResponseRequest {
    pub response: String, // Fixed: use String instead of enum
    pub response_notes: Option<String>,
}

// User Shift Schedules
pub async fn create_user_schedule(
    ctx: UserContext,
    input: Json<UserShiftScheduleInput>,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = input.user_id;

    ctx.requires_same_user(user_id)?;

    let schedule = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            schedule_repo::create_user_schedule(tx, input.into_inner())
                .await
                .map_err(AppError::from)
        })
    })
    .await?;

    // Cache invalidation for schedule creation - affects schedules and users
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(user_id),
                resource_id: Some(schedule.id),
            },
        )
        .await;

    Ok(ApiResponse::success(schedule))
}

pub async fn get_user_schedule(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    ctx.requires_same_user(user_id)?;

    let schedule = schedule_repo::get_user_schedule(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get user schedule: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("User schedule not found for user ID: {}", user_id))
        })?;

    // Cache read operation for schedule retrieval
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(user_id),
                resource_id: Some(schedule.id),
            },
        )
        .await;

    Ok(ApiResponse::success(schedule))
}

pub async fn update_user_schedule(
    path: Path<Uuid>,
    ctx: UserContext,
    input: Json<UserShiftScheduleInput>,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    ctx.requires_same_user(user_id)?;

    let schedule = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            schedule_repo::update_user_schedule(tx, user_id, input.into_inner())
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("User schedule not found for user ID: {}", user_id))
                })
        })
    })
    .await?;

    // Cache invalidation for schedule update - affects schedules and users
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(user_id),
                resource_id: Some(schedule.id),
            },
        )
        .await;

    Ok(ApiResponse::success(schedule))
}

pub async fn delete_user_schedule(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    ctx.requires_manager()?;

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            schedule_repo::delete_user_schedule(tx, user_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("User schedule not found for user ID: {}", user_id))
                })
        })
    })
    .await?;

    // Cache invalidation for schedule deletion - affects schedules and users
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(user_id),
                resource_id: None, // No specific resource ID for deletion
            },
        )
        .await;

    Ok(ApiResponse::success_message(
        "User schedule deleted successfully",
    ))
}

// Shift Assignments
pub async fn create_shift_assignment(
    ctx: UserContext,
    input: Json<ShiftAssignmentInput>,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;
    let user_id = ctx.user_id();

    let assignment = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            schedule_repo::create_shift_assignment(tx, user_id, input.into_inner())
                .await
                .map_err(AppError::from)
        })
    })
    .await?;

    // Cache invalidation for assignment creation - affects schedules, shifts, assignments
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(assignment.user_id),
                resource_id: Some(assignment.id),
            },
        )
        .await;

    Ok(ApiResponse::success(assignment))
}

pub async fn get_shift_assignment(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let assignment_id = path.into_inner();

    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get shift assignment: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))?;

    ctx.requires_same_user(assignment.user_id)?;

    // Cache assignment retrieval - affects assignments
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(assignment.user_id),
                resource_id: Some(assignment.id),
            },
        )
        .await;

    Ok(ApiResponse::success(assignment))
}

pub async fn get_shift_assignments_by_shift(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let shift_id = path.into_inner();

    let assignments = schedule_repo::get_shift_assignments_by_shift(shift_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get shift assignments: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Cache assignments by shift - affects assignments, shifts
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: None,
                resource_id: Some(shift_id),
            },
        )
        .await;

    Ok(ApiResponse::success(assignments))
}

pub async fn get_shift_assignments_by_user(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    ctx.requires_same_user(user_id)?;

    let assignments = schedule_repo::get_shift_assignments_by_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get user assignments: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Cache assignments by user - affects assignments, users
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(assignments))
}

pub async fn get_pending_assignments_for_user(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    ctx.requires_same_user(user_id)?;

    let assignments = schedule_repo::get_pending_assignments_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get pending assignments: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Cache pending assignments - affects assignments, users
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(assignments))
}

pub async fn respond_to_assignment(
    path: Path<Uuid>,
    ctx: UserContext,
    input: Json<AssignmentResponseRequest>,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let assignment_id = path.into_inner();

    // get the assignment first to check ownership
    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get assignment: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))?;

    ctx.requires_same_user(assignment.user_id)?;

    let updated_assignment = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            schedule_repo::respond_to_assignment(
                tx,
                assignment_id,
                input.response.clone(),
                input.response_notes.clone(),
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))
        })
    })
    .await?;

    // Cache invalidation for assignment response - affects assignments, schedules, shifts
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(assignment.user_id),
                resource_id: Some(assignment_id),
            },
        )
        .await;

    Ok(ApiResponse::success(updated_assignment))
}

pub async fn cancel_assignment(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let assignment_id = path.into_inner();

    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(|e| {
            log::error!("Failed to cancel assignment: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))?;

    // Cache invalidation for assignment cancellation - affects assignments, schedules, shifts
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(assignment.user_id),
                resource_id: Some(assignment_id),
            },
        )
        .await;

    Ok(ApiResponse::success(assignment))
}

pub async fn expire_overdue_assignments(
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_admin()?;

    let expired_assignments = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            schedule_repo::expire_overdue_assignments(tx)
                .await
                .map_err(AppError::from)
        })
    })
    .await?;

    // Cache invalidation for assignment expiration - affects assignments, schedules
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: None,
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(expired_assignments))
}

pub async fn get_user_shift_suggestions(
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let suggestions = schedule_repo::get_user_shift_suggestions(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get user shift suggestions: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Cache shift suggestions - affects shifts, schedules, users
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(suggestions))
}
