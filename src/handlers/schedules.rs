use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::{
    models::{AssignmentResponse, ShiftAssignmentInput, UserShiftScheduleInput},
    repositories::schedule as schedule_repo,
};
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::user_context::extract_context;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentResponseRequest {
    pub response: AssignmentResponse,
    pub response_notes: Option<String>,
}

// User Shift Schedules
pub async fn create_user_schedule(
    input: web::Json<UserShiftScheduleInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = input.user_id;

    user_context.requires_same_user(user_id)?;

    let schedule = schedule_repo::create_user_schedule(input.into_inner())
        .await
        .map_err(|e| {
            log::error!("Failed to create user schedule: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(schedule))
}

pub async fn get_user_schedule(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = path.into_inner();

    user_context.requires_same_user(user_id)?;

    let schedule = schedule_repo::get_user_schedule(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get user schedule: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("User schedule not found for user ID: {}", user_id))
        })?;

    Ok(ApiResponse::success(schedule))
}

pub async fn update_user_schedule(
    path: web::Path<Uuid>,
    input: web::Json<UserShiftScheduleInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = path.into_inner();

    user_context.requires_same_user(user_id)?;

    let schedule = schedule_repo::update_user_schedule(user_id, input.into_inner())
        .await
        .map_err(|e| {
            log::error!("Failed to update user schedule: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("User schedule not found for user ID: {}", user_id))
        })?;

    Ok(ApiResponse::success(schedule))
}

pub async fn delete_user_schedule(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = path.into_inner();

    user_context.requires_manager()?;

    schedule_repo::delete_user_schedule(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to delete user schedule: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("User schedule not found for user ID: {}", user_id))
        })?;

    Ok(ApiResponse::success_message(
        "User schedule deleted successfully",
    ))
}

// Shift Assignments
pub async fn create_shift_assignment(
    input: web::Json<ShiftAssignmentInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let assignment =
        schedule_repo::create_shift_assignment(user_context.user_id(), input.into_inner())
            .await
            .map_err(|e| {
                log::error!("Failed to create shift assignment: {}", e);
                AppError::DatabaseError(e)
            })?;

    Ok(ApiResponse::success(assignment))
}

pub async fn get_shift_assignment(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let assignment_id = path.into_inner();

    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get shift assignment: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))
        .unwrap();

    user_context.requires_same_user(assignment.user_id)?;

    Ok(ApiResponse::success(assignment))
}

pub async fn get_shift_assignments_by_shift(
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let shift_id = path.into_inner();

    let assignments = schedule_repo::get_shift_assignments_by_shift(shift_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get shift assignments: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(assignments))
}

pub async fn get_shift_assignments_by_user(
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = path.into_inner();

    user_context.requires_same_user(user_id)?;

    let assignments = schedule_repo::get_shift_assignments_by_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get user assignments: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(assignments))
}

pub async fn get_pending_assignments_for_user(
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = path.into_inner();

    user_context.requires_same_user(user_id)?;

    let assignments = schedule_repo::get_pending_assignments_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get pending assignments: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(assignments))
}

pub async fn respond_to_assignment(
    path: web::Path<Uuid>,
    input: web::Json<AssignmentResponseRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let assignment_id = path.into_inner();

    // get the assignment first to check ownership
    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get assignment: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))
        .unwrap();

    user_context.requires_same_user(assignment.user_id)?;

    let updated_assignment = schedule_repo::respond_to_assignment(
        assignment_id,
        input.response.clone(),
        input.response_notes.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Failed to respond to assignment: {}", e);
        AppError::DatabaseError(e)
    })?
    .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))?;

    Ok(ApiResponse::success(updated_assignment))
}

pub async fn cancel_assignment(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let assignment_id = path.into_inner();

    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(|e| {
            log::error!("Failed to cancel assignment: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift assignment not found".to_string()))?;

    Ok(ApiResponse::success(assignment))
}

pub async fn expire_overdue_assignments(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_admin()?;

    let expired_assignments = schedule_repo::expire_overdue_assignments()
        .await
        .map_err(|e| {
            log::error!("Failed to expire overdue assignments: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(expired_assignments))
}

pub async fn get_user_shift_suggestions(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();

    let suggestions = schedule_repo::get_user_shift_suggestions(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get user shift suggestions: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(suggestions))
}
