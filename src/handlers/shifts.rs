use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::database::shift_repository::ShiftRepository;
use crate::database::models::{ShiftInput, ShiftStatus};
use crate::auth::Claims;
use crate::handlers::admin::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct ShiftQuery {
    pub location_id: Option<i64>,
    pub team_id: Option<i64>,
    pub user_id: Option<i64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignShiftRequest {
    pub user_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftStatusRequest {
    pub status: String,
}

// Shift handlers
pub async fn create_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    input: web::Json<ShiftInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    match shift_repo.create_shift(input.into_inner()).await {
        Ok(shift) => Ok(HttpResponse::Created().json(ApiResponse::success(shift))),
        Err(err) => {
            log::error!("Error creating shift: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create shift")))
        }
    }
}

pub async fn get_shifts(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    query: web::Query<ShiftQuery>,
) -> Result<HttpResponse> {
    let shifts = if let Some(user_id) = query.user_id {
        // Users can only see their own shifts unless they are admin/manager
        if !claims.is_admin() && !claims.is_manager() && user_id != claims.user_id().parse::<i64>().unwrap_or(-1) {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        shift_repo.get_shifts_by_user(user_id).await
    } else if let Some(team_id) = query.team_id {
        shift_repo.get_shifts_by_team(team_id).await
    } else if let Some(location_id) = query.location_id {
        shift_repo.get_shifts_by_location(location_id).await
    } else if let (Some(start_date), Some(end_date)) = (query.start_date, query.end_date) {
        shift_repo.get_shifts_by_date_range(start_date, end_date, query.location_id).await
    } else if query.status.as_deref() == Some("open") {
        shift_repo.get_open_shifts(query.location_id).await
    } else {
        // For general queries, only admin/manager can see all shifts
        if !claims.is_admin() && !claims.is_manager() {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        shift_repo.get_open_shifts(query.location_id).await
    };

    match shifts {
        Ok(shifts) => Ok(HttpResponse::Ok().json(ApiResponse::success(shifts))),
        Err(err) => {
            log::error!("Error fetching shifts: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch shifts")))
        }
    }
}

pub async fn get_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();

    match shift_repo.get_shift_by_id(shift_id).await {
        Ok(Some(shift)) => {
            // Check if user has permission to view this shift
            if !claims.is_admin() && !claims.is_manager() {
                if let Some(assigned_user_id) = shift.assigned_user_id {
                    if assigned_user_id != claims.user_id().parse::<i64>().unwrap_or(-1) {
                        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
                    }
                }
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(shift)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error fetching shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch shift")))
        }
    }
}

pub async fn update_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
    input: web::Json<ShiftInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let shift_id = path.into_inner();

    match shift_repo.update_shift(shift_id, input.into_inner()).await {
        Ok(Some(shift)) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error updating shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update shift")))
        }
    }
}

pub async fn assign_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
    input: web::Json<AssignShiftRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let shift_id = path.into_inner();

    match shift_repo.assign_shift(shift_id, input.user_id).await {
        Ok(Some(shift)) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error assigning shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to assign shift")))
        }
    }
}

pub async fn unassign_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let shift_id = path.into_inner();

    match shift_repo.unassign_shift(shift_id).await {
        Ok(Some(shift)) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error unassigning shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to unassign shift")))
        }
    }
}

pub async fn update_shift_status(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
    input: web::Json<UpdateShiftStatusRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let shift_id = path.into_inner();
    
    let status = match input.status.parse::<ShiftStatus>() {
        Ok(status) => status,
        Err(_) => return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid shift status"))),
    };

    match shift_repo.update_shift_status(shift_id, status).await {
        Ok(Some(shift)) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error updating shift status {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update shift status")))
        }
    }
}

pub async fn delete_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let shift_id = path.into_inner();

    match shift_repo.delete_shift(shift_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success("Shift deleted successfully"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error deleting shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete shift")))
        }
    }
}

// Employee shift claiming (for future shift swapping feature)
pub async fn claim_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();
    let user_id = claims.user_id().parse::<i64>().unwrap_or(-1);

    if user_id == -1 {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid user ID")));
    }

    // Check if shift exists and is open
    match shift_repo.get_shift_by_id(shift_id).await {
        Ok(Some(shift)) => {
            if shift.assigned_user_id.is_some() {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Shift is already assigned")));
            }
        }
        Ok(None) => return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error fetching shift {}: {}", shift_id, err);
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch shift")));
        }
    }

    match shift_repo.assign_shift(shift_id, user_id).await {
        Ok(Some(shift)) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error claiming shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to claim shift")))
        }
    }
}
