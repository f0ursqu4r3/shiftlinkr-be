use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

use crate::database::models::{ShiftClaimInput, ShiftInput, ShiftStatus};
use crate::database::models::activity::{Action};
use crate::database::repositories::shift::ShiftRepository;
use crate::database::repositories::shift_claim::ShiftClaimRepository;
use crate::database::repositories::company::CompanyRepository;
use crate::handlers::admin::ApiResponse;
use crate::services::auth::Claims;
use crate::services::activity_logger::ActivityLogger;

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
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    input: web::Json<ShiftInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_input = input.into_inner();
    let location_id = shift_input.location_id;
    let team_id = shift_input.team_id;
    let start_time = shift_input.start_time;
    let end_time = shift_input.end_time;

    match shift_repo.create_shift(shift_input).await {
        Ok(shift) => {
            // Log shift creation activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("location_id".to_string(), serde_json::Value::Number(serde_json::Number::from(location_id)));
                if let Some(team_id) = team_id {
                    metadata.insert("team_id".to_string(), serde_json::Value::Number(serde_json::Number::from(team_id)));
                }
                metadata.insert("start_time".to_string(), serde_json::Value::String(start_time.to_string()));
                metadata.insert("end_time".to_string(), serde_json::Value::String(end_time.to_string()));
                
                if let Err(e) = activity_logger
                    .log_shift_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        shift.id,
                        Action::CREATED,
                        format!("Shift created for location {} from {} to {}", location_id, start_time, end_time),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log shift creation activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Created().json(ApiResponse::success(shift)))
        }
        Err(err) => {
            log::error!("Error creating shift: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create shift")))
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
        if !claims.is_admin()
            && !claims.is_manager()
            && user_id != claims.user_id().parse::<i64>().unwrap_or(-1)
        {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        shift_repo.get_shifts_by_user(user_id).await
    } else if let Some(team_id) = query.team_id {
        shift_repo.get_shifts_by_team(team_id).await
    } else if let Some(location_id) = query.location_id {
        shift_repo.get_shifts_by_location(location_id).await
    } else if let (Some(start_date), Some(end_date)) = (query.start_date, query.end_date) {
        shift_repo
            .get_shifts_by_date_range(
                start_date.naive_utc(),
                end_date.naive_utc(),
                query.location_id,
            )
            .await
    } else if query.status.as_deref() == Some("open") {
        if let Some(location_id) = query.location_id {
            shift_repo.get_open_shifts_by_location(location_id).await
        } else {
            shift_repo.get_open_shifts().await
        }
    } else {
        // For general queries, only admin/manager can see all shifts
        if !claims.is_admin() && !claims.is_manager() {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        if let Some(location_id) = query.location_id {
            shift_repo.get_open_shifts_by_location(location_id).await
        } else {
            shift_repo.get_open_shifts().await
        }
    };

    match shifts {
        Ok(shifts) => Ok(HttpResponse::Ok().json(ApiResponse::success(shifts))),
        Err(err) => {
            log::error!("Error fetching shifts: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch shifts")))
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
                        return Ok(HttpResponse::Forbidden()
                            .json(ApiResponse::<()>::error("Insufficient permissions")));
                    }
                }
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(shift)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error fetching shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch shift")))
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
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();

    match shift_repo.update_shift(shift_id, input.into_inner()).await {
        Ok(Some(shift)) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error updating shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update shift")))
        }
    }
}

pub async fn assign_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<i64>,
    input: web::Json<AssignShiftRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();
    let assigned_user_id = input.user_id;

    match shift_repo.assign_shift(shift_id, assigned_user_id).await {
        Ok(Some(shift)) => {
            // Log shift assignment activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("assigned_user_id".to_string(), serde_json::Value::Number(serde_json::Number::from(assigned_user_id)));
                metadata.insert("location_id".to_string(), serde_json::Value::Number(serde_json::Number::from(shift.location_id)));
                if let Some(team_id) = shift.team_id {
                    metadata.insert("team_id".to_string(), serde_json::Value::Number(serde_json::Number::from(team_id)));
                }
                
                if let Err(e) = activity_logger
                    .log_shift_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        shift_id,
                        Action::ASSIGNED,
                        format!("Shift assigned to user {}", assigned_user_id),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log shift assignment activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(shift)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error assigning shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to assign shift")))
        }
    }
}

pub async fn unassign_shift(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<i64>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();

    // Get shift details before unassigning for logging
    let previous_assignment = shift_repo.get_shift_by_id(shift_id).await.ok().flatten();

    match shift_repo.unassign_shift(shift_id).await {
        Ok(Some(shift)) => {
            // Log shift unassignment activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(&claims.sub)
                .await
            {
                let mut metadata = HashMap::new();
                if let Some(prev_shift) = previous_assignment {
                    if let Some(prev_user_id) = prev_shift.assigned_user_id {
                        metadata.insert("previously_assigned_user_id".to_string(), serde_json::Value::Number(serde_json::Number::from(prev_user_id)));
                    }
                }
                metadata.insert("location_id".to_string(), serde_json::Value::Number(serde_json::Number::from(shift.location_id)));
                if let Some(team_id) = shift.team_id {
                    metadata.insert("team_id".to_string(), serde_json::Value::Number(serde_json::Number::from(team_id)));
                }
                
                if let Err(e) = activity_logger
                    .log_shift_activity(
                        company.id,
                        Some(claims.sub.parse().unwrap_or(0)),
                        shift_id,
                        Action::UNASSIGNED,
                        "Shift unassigned".to_string(),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log shift unassignment activity: {}", e);
                }
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(shift)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error unassigning shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to unassign shift")))
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
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();

    let status = match input.status.parse::<ShiftStatus>() {
        Ok(status) => status,
        Err(_) => {
            return Ok(
                HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid shift status"))
            )
        }
    };

    match shift_repo.update_shift_status(shift_id, status).await {
        Ok(Some(shift)) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error updating shift status {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update shift status")))
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
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();

    match shift_repo.delete_shift(shift_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success("Shift deleted successfully"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error deleting shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to delete shift")))
        }
    }
}

// Employee shift claiming with proper validation and workflow
pub async fn claim_shift(
    claims: Claims,
    _shift_repo: web::Data<ShiftRepository>,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();
    let user_id = claims.user_id();

    // Get shift information for validation
    let shift_info = match shift_claim_repo.get_shift_claim_info(shift_id).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found")))
        }
        Err(err) => {
            log::error!("Error fetching shift info {}: {}", shift_id, err);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to fetch shift information",
                )),
            );
        }
    };

    // Validate shift is claimable
    if shift_info.assigned_user_id.is_some() {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error("Shift is already assigned"))
        );
    }

    if shift_info.shift_status != "open" {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Shift is not available for claiming",
        )));
    }

    // Check if shift is too close to start time (must be at least 2 hours in advance)
    let now = Utc::now().naive_utc();
    let time_until_shift = shift_info.start_time - now;
    if time_until_shift.num_hours() < 2 {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Cannot claim shift less than 2 hours before start time",
        )));
    }

    // Check if user has already claimed this shift
    match shift_claim_repo
        .has_user_claimed_shift(shift_id, user_id)
        .await
    {
        Ok(true) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "You have already claimed this shift",
            )))
        }
        Ok(false) => {}
        Err(err) => {
            log::error!(
                "Error checking existing claim for shift {} user {}: {}",
                shift_id,
                user_id,
                err
            );
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to validate claim")));
        }
    }

    // Check if user is a team member (if shift has a team)
    if let Some(_team_id) = shift_info.team_id {
        match shift_claim_repo
            .is_user_team_member(shift_id, user_id)
            .await
        {
            Ok(true) => {}
            Ok(false) => {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "You are not a member of this shift's team",
                )))
            }
            Err(err) => {
                log::error!(
                    "Error checking team membership for shift {} user {}: {}",
                    shift_id,
                    user_id,
                    err
                );
                return Ok(
                    HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        "Failed to validate team membership",
                    )),
                );
            }
        }
    }

    // Create the shift claim
    let claim_input = ShiftClaimInput {
        shift_id,
        user_id: user_id.to_string(),
    };

    match shift_claim_repo.create_claim(&claim_input).await {
        Ok(claim) => {
            log::info!(
                "User {} claimed shift {} - claim ID: {}",
                user_id,
                shift_id,
                claim.id
            );
            Ok(HttpResponse::Created().json(ApiResponse::success(claim)))
        }
        Err(err) => {
            log::error!(
                "Error creating claim for shift {} user {}: {}",
                shift_id,
                user_id,
                err
            );
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create claim")))
        }
    }
}

// Get claims for a specific shift (managers/admins only)
pub async fn get_shift_claims(
    claims: Claims,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();

    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match shift_claim_repo.get_claims_by_shift(shift_id).await {
        Ok(claims) => Ok(HttpResponse::Ok().json(ApiResponse::success(claims))),
        Err(err) => {
            log::error!("Error fetching claims for shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch claims")))
        }
    }
}

// Get user's own claims
pub async fn get_my_claims(
    claims: Claims,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
) -> Result<HttpResponse> {
    let user_id = claims.user_id();

    match shift_claim_repo.get_claims_by_user(user_id).await {
        Ok(claims) => Ok(HttpResponse::Ok().json(ApiResponse::success(claims))),
        Err(err) => {
            log::error!("Error fetching claims for user {}: {}", user_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch claims")))
        }
    }
}

// Approve a shift claim (managers/admins only)
pub async fn approve_shift_claim(
    claims: Claims,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<i64>,
    approval_data: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    let claim_id = path.into_inner();
    let approver_id = claims.user_id();

    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    // Get the claim to approve
    let claim = match shift_claim_repo.get_claim_by_id(claim_id).await {
        Ok(Some(claim)) => claim,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Claim not found")))
        }
        Err(err) => {
            log::error!("Error fetching claim {}: {}", claim_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch claim")));
        }
    };

    // Approve the claim
    match shift_claim_repo
        .approve_claim(claim_id, approver_id, approval_data.notes.clone())
        .await
    {
        Ok(Some(approved_claim)) => {
            // Assign the shift to the user - need to parse user_id as i64
            let user_id_i64 = match claim.user_id.parse::<i64>() {
                Ok(id) => id,
                Err(_) => {
                    log::error!(
                        "Invalid user_id format in claim {}: {}",
                        claim_id,
                        claim.user_id
                    );
                    return Ok(HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Invalid user ID format")));
                }
            };

            match shift_repo.assign_shift(claim.shift_id, user_id_i64).await {
                Ok(Some(assigned_shift)) => {
                    // Cancel any other pending claims for this shift
                    let _ = shift_claim_repo
                        .cancel_pending_claims_for_shift(claim.shift_id)
                        .await;

                    log::info!(
                        "Approved claim {} for shift {} by user {}",
                        claim_id,
                        claim.shift_id,
                        approver_id
                    );
                    Ok(
                        HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                            "claim": approved_claim,
                            "shift": assigned_shift
                        }))),
                    )
                }
                Ok(None) => {
                    log::error!(
                        "Failed to assign shift {} after approving claim {}",
                        claim.shift_id,
                        claim_id
                    );
                    Ok(HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Failed to assign shift")))
                }
                Err(err) => {
                    log::error!(
                        "Error assigning shift {} after approving claim {}: {}",
                        claim.shift_id,
                        claim_id,
                        err
                    );
                    Ok(HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Failed to assign shift")))
                }
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Claim not found or already processed",
        ))),
        Err(err) => {
            log::error!("Error approving claim {}: {}", claim_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to approve claim")))
        }
    }
}

// Reject a shift claim (managers/admins only)
pub async fn reject_shift_claim(
    claims: Claims,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    path: web::Path<i64>,
    rejection_data: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    let claim_id = path.into_inner();
    let approver_id = claims.user_id();

    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match shift_claim_repo
        .reject_claim(claim_id, approver_id, rejection_data.notes.clone())
        .await
    {
        Ok(Some(rejected_claim)) => {
            log::info!("Rejected claim {} by user {}", claim_id, approver_id);
            Ok(HttpResponse::Ok().json(ApiResponse::success(rejected_claim)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Claim not found or already processed",
        ))),
        Err(err) => {
            log::error!("Error rejecting claim {}: {}", claim_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to reject claim")))
        }
    }
}

// Cancel a shift claim (by the user who made it)
pub async fn cancel_shift_claim(
    claims: Claims,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let claim_id = path.into_inner();
    let user_id = claims.user_id();

    match shift_claim_repo.cancel_claim(claim_id, user_id).await {
        Ok(Some(cancelled_claim)) => {
            log::info!("User {} cancelled claim {}", user_id, claim_id);
            Ok(HttpResponse::Ok().json(ApiResponse::success(cancelled_claim)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Claim not found or not cancellable",
        ))),
        Err(err) => {
            log::error!(
                "Error cancelling claim {} for user {}: {}",
                claim_id,
                user_id,
                err
            );
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to cancel claim")))
        }
    }
}

// Get pending claims for approval (managers/admins only)
pub async fn get_pending_claims(
    claims: Claims,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match shift_claim_repo.get_pending_claims().await {
        Ok(claims) => Ok(HttpResponse::Ok().json(ApiResponse::success(claims))),
        Err(err) => {
            log::error!("Error fetching pending claims: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch pending claims")))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}
