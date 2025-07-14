use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;

use crate::database::models::{AssignmentResponse, ShiftAssignmentInput, UserShiftScheduleInput};
use crate::database::repositories::schedule::ScheduleRepository;
use crate::handlers::admin::ApiResponse;
use crate::services::auth::Claims;

#[derive(Debug, Deserialize)]
pub struct AssignmentResponseRequest {
    pub response: AssignmentResponse,
    pub response_notes: Option<String>,
}

// User Shift Schedules
pub async fn create_user_schedule(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    input: web::Json<UserShiftScheduleInput>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Users can only create their own schedule, admins can create any
    if !claims.is_admin() && claims.user_id() != input.user_id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Can only manage your own schedule",
        )));
    }

    match schedule_repo.create_user_schedule(input.into_inner()).await {
        Ok(schedule) => Ok(HttpResponse::Created().json(ApiResponse::success(schedule))),
        Err(e) => {
            log::error!("Failed to create user schedule: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create user schedule")))
        }
    }
}

pub async fn get_user_schedule(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<String>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only view their own schedule, admins/managers can view any
    if !claims.is_admin() && !claims.is_manager() && claims.user_id() != user_id {
        return Ok(HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("Can only view your own schedule")));
    }

    match schedule_repo.get_user_schedule(&user_id).await {
        Ok(Some(schedule)) => Ok(HttpResponse::Ok().json(ApiResponse::success(schedule))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("User schedule not found")))
        }
        Err(e) => {
            log::error!("Failed to get user schedule: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get user schedule")))
        }
    }
}

pub async fn update_user_schedule(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<String>,
    input: web::Json<UserShiftScheduleInput>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only update their own schedule, admins can update any
    if !claims.is_admin() && claims.user_id() != user_id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Can only manage your own schedule",
        )));
    }

    match schedule_repo
        .update_user_schedule(&user_id, input.into_inner())
        .await
    {
        Ok(Some(schedule)) => Ok(HttpResponse::Ok().json(ApiResponse::success(schedule))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("User schedule not found")))
        }
        Err(e) => {
            log::error!("Failed to update user schedule: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update user schedule")))
        }
    }
}

pub async fn delete_user_schedule(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<String>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only delete their own schedule, admins can delete any
    if !claims.is_admin() && claims.user_id() != user_id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Can only manage your own schedule",
        )));
    }

    match schedule_repo.delete_user_schedule(&user_id).await {
        Ok(true) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success("User schedule deleted successfully")))
        }
        Ok(false) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("User schedule not found")))
        }
        Err(e) => {
            log::error!("Failed to delete user schedule: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to delete user schedule")))
        }
    }
}

// Shift Assignments
pub async fn create_shift_assignment(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    input: web::Json<ShiftAssignmentInput>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Manager access required"))
        );
    }

    match schedule_repo
        .create_shift_assignment(input.into_inner())
        .await
    {
        Ok(assignment) => Ok(HttpResponse::Created().json(ApiResponse::success(assignment))),
        Err(e) => {
            log::error!("Failed to create shift assignment: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create shift assignment",
                )),
            )
        }
    }
}

pub async fn get_shift_assignment(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<i64>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let assignment_id = path.into_inner();

    match schedule_repo.get_shift_assignment(assignment_id).await {
        Ok(Some(assignment)) => {
            // Users can only view their own assignments, admins/managers can view any
            if !claims.is_admin() && !claims.is_manager() && claims.user_id() != assignment.user_id
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "Can only view your own assignments",
                )));
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(assignment)))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Shift assignment not found")))
        }
        Err(e) => {
            log::error!("Failed to get shift assignment: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get shift assignment")))
        }
    }
}

pub async fn get_shift_assignments_by_shift(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<i64>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Manager access required"))
        );
    }

    let shift_id = path.into_inner();

    match schedule_repo.get_shift_assignments_by_shift(shift_id).await {
        Ok(assignments) => Ok(HttpResponse::Ok().json(ApiResponse::success(assignments))),
        Err(e) => {
            log::error!("Failed to get shift assignments: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get shift assignments")))
        }
    }
}

pub async fn get_shift_assignments_by_user(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<String>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only view their own assignments, admins/managers can view any
    if !claims.is_admin() && !claims.is_manager() && claims.user_id() != user_id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Can only view your own assignments",
        )));
    }

    match schedule_repo.get_shift_assignments_by_user(&user_id).await {
        Ok(assignments) => Ok(HttpResponse::Ok().json(ApiResponse::success(assignments))),
        Err(e) => {
            log::error!("Failed to get user assignments: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get user assignments")))
        }
    }
}

pub async fn get_pending_assignments_for_user(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<String>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only view their own pending assignments, admins/managers can view any
    if !claims.is_admin() && !claims.is_manager() && claims.user_id() != user_id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Can only view your own assignments",
        )));
    }

    match schedule_repo
        .get_pending_assignments_for_user(&user_id)
        .await
    {
        Ok(assignments) => Ok(HttpResponse::Ok().json(ApiResponse::success(assignments))),
        Err(e) => {
            log::error!("Failed to get pending assignments: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to get pending assignments",
                )),
            )
        }
    }
}

pub async fn respond_to_assignment(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<i64>,
    input: web::Json<AssignmentResponseRequest>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let assignment_id = path.into_inner();

    // First get the assignment to check ownership
    match schedule_repo.get_shift_assignment(assignment_id).await {
        Ok(Some(assignment)) => {
            // Users can only respond to their own assignments
            if claims.user_id() != assignment.user_id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "Can only respond to your own assignments",
                )));
            }

            match schedule_repo
                .respond_to_assignment(
                    assignment_id,
                    input.response.clone(),
                    input.response_notes.clone(),
                )
                .await
            {
                Ok(Some(updated_assignment)) => {
                    Ok(HttpResponse::Ok().json(ApiResponse::success(updated_assignment)))
                }
                Ok(None) => {
                    Ok(HttpResponse::NotFound()
                        .json(ApiResponse::<()>::error("Assignment not found")))
                }
                Err(e) => {
                    log::error!("Failed to respond to assignment: {}", e);
                    Ok(HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Failed to respond to assignment")))
                }
            }
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Assignment not found")))
        }
        Err(e) => {
            log::error!("Failed to get assignment: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get assignment")))
        }
    }
}

pub async fn cancel_assignment(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<i64>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Manager access required"))
        );
    }

    let assignment_id = path.into_inner();

    match schedule_repo.cancel_assignment(assignment_id).await {
        Ok(Some(assignment)) => Ok(HttpResponse::Ok().json(ApiResponse::success(assignment))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Assignment not found")))
        }
        Err(e) => {
            log::error!("Failed to cancel assignment: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to cancel assignment")))
        }
    }
}

pub async fn expire_overdue_assignments(
    claims: Claims,
    schedule_repo: web::Data<ScheduleRepository>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin
    if !claims.is_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"))
        );
    }

    match schedule_repo.expire_overdue_assignments().await {
        Ok(expired_assignments) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(expired_assignments)))
        }
        Err(e) => {
            log::error!("Failed to expire overdue assignments: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to expire overdue assignments",
                )),
            )
        }
    }
}
