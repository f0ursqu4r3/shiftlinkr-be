// TODO: refactor
use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::database::models::activity::Action;
use crate::database::models::{ShiftAssignmentInput, ShiftClaimInput, ShiftInput, ShiftStatus};
use crate::database::repositories::company::CompanyRepository;
use crate::database::repositories::schedule::ScheduleRepository;
use crate::database::repositories::shift::ShiftRepository;
use crate::database::repositories::shift_claim::ShiftClaimRepository;
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::activity_logger::ActivityLogger;
use crate::services::user_context::AsyncUserContext;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftQuery {
    pub location_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignShiftRequest {
    pub user_id: Uuid,
    pub acceptance_deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectAssignShiftRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftStatusRequest {
    pub status: String,
}

// Shift handlers
pub async fn create_shift(
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: web::Data<ActivityLogger>,
    shift_repo: web::Data<ShiftRepository>,
    input: web::Json<ShiftInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    user_context.requires_manager()?;

    let shift_input = input.into_inner();

    let shift = shift_repo.create_shift(shift_input).await.map_err(|e| {
        log::error!("Failed to create shift: {}", e);
        AppError::DatabaseError(e)
    })?;

    // Log shift creation activity
    let metadata = ActivityLogger::metadata(vec![
        ("location_id", shift.location_id.to_string()),
        (
            "team_id",
            shift
                .team_id
                .map_or("None".to_string(), |id| id.to_string()),
        ),
        ("start_time", shift.start_time.to_string()),
        ("end_time", shift.end_time.to_string()),
    ]);
    if let Err(e) = activity_logger
        .log_shift_activity(
            user_context.company_id().unwrap_or_default(),
            Some(user_context.user.id),
            shift.id,
            Action::CREATED,
            "Shift created".to_string(),
            Some(metadata),
            &req,
        )
        .await
    {
        log::warn!("Failed to log shift creation activity: {}", e);
    }

    Ok(ApiResponse::created(shift))
}

pub async fn get_shifts(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    query: web::Query<ShiftQuery>,
) -> Result<HttpResponse> {
    // Check if user has manager/admin permissions
    let has_manager_permissions = user_context.is_manager_or_admin();

    let shifts = if let Some(user_id) = query.user_id {
        // Users can only see their own shifts unless they are admin/manager
        if !has_manager_permissions && !user_context.can_access_user_resource(user_id) {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        shift_repo.find_shifts_by_user(user_id).await
    } else if let Some(team_id) = query.team_id {
        shift_repo.find_by_team_id(team_id).await
    } else if let Some(location_id) = query.location_id {
        shift_repo.find_by_location_id(location_id).await
    } else if let (Some(start_date), Some(end_date)) = (query.start_date, query.end_date) {
        shift_repo
            .find_by_date_range(start_date, end_date, query.location_id)
            .await
    } else if query.status.as_deref() == Some("open") {
        if let Some(location_id) = query.location_id {
            shift_repo.find_open_shifts_by_location(location_id).await
        } else {
            shift_repo.find_open_shifts().await
        }
    } else {
        // For general queries, only admin/manager can see all shifts
        if !has_manager_permissions {
            return Err(AppError::Forbidden("Insufficient permissions".to_string()).into());
        }

        // Get shifts for user's company if they have one
        if let Some(company_id) = user_context.company_id() {
            shift_repo.find_by_company_id(company_id).await
        } else {
            shift_repo.find_open_shifts().await
        }
    };

    match shifts {
        Ok(shifts) => Ok(ApiResponse::success(shifts)),
        Err(err) => {
            log::error!(
                "Error fetching shifts for user {}: {}",
                user_context.user_id(),
                err
            );
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch shifts")))
        }
    }
}

pub async fn get_shift(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();

    match shift_repo.find_by_id(shift_id).await {
        Ok(Some(shift)) => {
            // Check if user has permission to view this shift
            // Allow viewing if user is admin/manager or if they have assignment for this shift
            if !user_context.is_manager_or_admin() {
                // For now, we'll be permissive for regular employees
                // In future, could add check if user has assignment for this shift
            }
            Ok(ApiResponse::success(shift))
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
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<Uuid>,
    input: web::Json<ShiftInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();

    match shift_repo.update_shift(shift_id, input.into_inner()).await {
        Ok(Some(shift)) => Ok(ApiResponse::success(shift)),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error updating shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update shift")))
        }
    }
}

pub async fn assign_shift(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    schedule_repo: web::Data<ScheduleRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<Uuid>,
    input: web::Json<AssignShiftRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();
    let assigned_user_id = input.user_id;
    let acceptance_deadline = input.acceptance_deadline;

    let user_id = user_context.user_id();

    // Create shift assignment using schedule repository
    let assignment_input = ShiftAssignmentInput {
        shift_id,
        user_id: assigned_user_id,
        acceptance_deadline,
    };

    match schedule_repo
        .create_shift_assignment(user_id, assignment_input)
        .await
    {
        Ok(assignment) => {
            // Update shift status to assigned
            match shift_repo.assign_shift(shift_id, assigned_user_id).await {
                Ok(Some(shift)) => {
                    // Log shift assignment activity
                    if let Ok(Some(company)) =
                        company_repo.get_primary_company_for_user(user_id).await
                    {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "assigned_user_id".to_string(),
                            serde_json::Value::String(assigned_user_id.to_string()),
                        );
                        metadata.insert(
                            "assignment_id".to_string(),
                            serde_json::Value::String(assignment.id.to_string()),
                        );
                        metadata.insert(
                            "location_id".to_string(),
                            serde_json::Value::String(shift.location_id.to_string()),
                        );
                        if let Some(team_id) = shift.team_id {
                            metadata.insert(
                                "team_id".to_string(),
                                serde_json::Value::String(team_id.to_string()),
                            );
                        }

                        if let Err(e) = activity_logger
                            .log_shift_activity(
                                company.id,
                                Some(user_id),
                                shift_id,
                                Action::ASSIGNED,
                                format!(
                                    "Shift assigned to user {} via assignment system",
                                    assigned_user_id
                                ),
                                Some(metadata),
                                &req,
                            )
                            .await
                        {
                            log::warn!("Failed to log shift assignment activity: {}", e);
                        }
                    }

                    Ok(ApiResponse::success(json!({
                        "shift": shift,
                        "assignment": assignment
                    })))
                }
                Ok(None) => {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found")))
                }
                Err(err) => {
                    log::error!(
                        "Error updating shift status after assignment creation {}: {}",
                        shift_id,
                        err
                    );
                    Ok(HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Failed to update shift status")))
                }
            }
        }
        Err(err) => {
            log::error!("Error creating shift assignment {}: {}", shift_id, err);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create shift assignment",
                )),
            )
        }
    }
}

pub async fn unassign_shift(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();
    let user_id = user_context.user_id();

    match shift_repo.unassign_shift(shift_id).await {
        Ok(Some(shift)) => {
            // Log shift unassignment activity
            if let Ok(Some(company)) = company_repo.get_primary_company_for_user(user_id).await {
                let mut metadata = HashMap::new();
                // Note: In the new system, we would get previous assignment info from schedule repository
                // For now, we'll just log basic info
                metadata.insert(
                    "location_id".to_string(),
                    serde_json::Value::String(shift.location_id.to_string()),
                );
                if let Some(team_id) = shift.team_id {
                    metadata.insert(
                        "team_id".to_string(),
                        serde_json::Value::String(team_id.to_string()),
                    );
                }

                if let Err(e) = activity_logger
                    .log_shift_activity(
                        company.id,
                        Some(user_id),
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

            Ok(ApiResponse::success(shift))
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
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<Uuid>,
    input: web::Json<UpdateShiftStatusRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
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
        Ok(Some(shift)) => Ok(ApiResponse::success(shift)),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error updating shift status {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update shift status")))
        }
    }
}

pub async fn delete_shift(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let shift_id = path.into_inner();

    match shift_repo.delete_shift(shift_id).await {
        Ok(true) => Ok(HttpResponse::NoContent().finish()),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Shift not found"))),
        Err(err) => {
            log::error!("Error deleting shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to delete shift")))
        }
    }
}

// Get shift assignments for a specific shift (managers/admins only)
pub async fn get_shift_assignments(
    AsyncUserContext(user_context): AsyncUserContext,
    schedule_repo: web::Data<ScheduleRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();

    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match schedule_repo.get_shift_assignments_by_shift(shift_id).await {
        Ok(assignments) => Ok(ApiResponse::success(assignments)),
        Err(err) => {
            log::error!("Error fetching assignments for shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch assignments")))
        }
    }
}

// Get user's pending assignments
pub async fn get_my_pending_assignments(
    AsyncUserContext(user_context): AsyncUserContext,
    schedule_repo: web::Data<ScheduleRepository>,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();

    match schedule_repo
        .get_pending_assignments_for_user(user_id)
        .await
    {
        Ok(assignments) => Ok(ApiResponse::success(assignments)),
        Err(err) => {
            log::error!(
                "Error fetching pending assignments for user {}: {}",
                user_id,
                err
            );
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to fetch pending assignments",
                )),
            )
        }
    }
}

// Respond to a shift assignment
pub async fn respond_to_assignment(
    AsyncUserContext(user_context): AsyncUserContext,
    schedule_repo: web::Data<ScheduleRepository>,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<Uuid>,
    response_data: web::Json<AssignmentResponseRequest>,
) -> Result<HttpResponse> {
    let assignment_id = path.into_inner();

    // Note: Additional validation could be added here to ensure the user
    // is responding to their own assignment
    let _user_id = user_context.user_id(); // Available for future validation

    // Parse response
    let response = match response_data.response.as_str() {
        "accepted" => crate::database::models::AssignmentResponse::Accept,
        "declined" => crate::database::models::AssignmentResponse::Decline,
        _ => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Invalid response. Must be 'accepted' or 'declined'",
            )))
        }
    };

    let is_accepted = matches!(
        response,
        crate::database::models::AssignmentResponse::Accept
    );

    match schedule_repo
        .respond_to_assignment(assignment_id, response, response_data.notes.clone())
        .await
    {
        Ok(Some(assignment)) => {
            // If accepted, update shift status
            if is_accepted {
                if let Err(e) = shift_repo
                    .update_shift_status(
                        assignment.shift_id,
                        crate::database::models::ShiftStatus::Assigned,
                    )
                    .await
                {
                    log::warn!("Failed to update shift status after acceptance: {}", e);
                }
            }

            Ok(ApiResponse::success(assignment))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Assignment not found or not yours to respond to",
        ))),
        Err(err) => {
            log::error!("Error responding to assignment {}: {}", assignment_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to respond to assignment")))
        }
    }
}

// Employee shift claiming with proper validation and workflow
pub async fn claim_shift(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_repo: web::Data<ShiftRepository>,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();
    let user_id = user_context.user_id();

    // Get shift information for validation
    let shift_info = match shift_repo.find_by_id(shift_id).await {
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
    // In the new proposal system, we check if shift is assigned via status
    if !matches!(shift_info.status, ShiftStatus::Open) {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Shift is not available for claiming",
        )));
    }

    // Check if shift is too close to start time (must be at least 2 hours in advance)
    let now = Utc::now();
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
    let claim_input = ShiftClaimInput { shift_id, user_id };

    match shift_claim_repo.create_claim(&claim_input).await {
        Ok(claim) => {
            log::info!(
                "User {} claimed shift {} - claim ID: {}",
                user_id,
                shift_id,
                claim.id
            );

            // Log shift claim activity
            if let Ok(Some(company)) = company_repo
                .get_primary_company_for_user(user_context.user.id)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "shift_id".to_string(),
                    serde_json::Value::String(shift_id.to_string()),
                );
                metadata.insert(
                    "claiming_user_id".to_string(),
                    serde_json::Value::String(user_id.to_string()),
                );
                if let Some(team_id) = shift_info.team_id {
                    metadata.insert(
                        "team_id".to_string(),
                        serde_json::Value::String(team_id.to_string()),
                    );
                }
                metadata.insert(
                    "start_time".to_string(),
                    serde_json::Value::String(shift_info.start_time.to_string()),
                );

                if let Err(e) = activity_logger
                    .log_shift_activity(
                        company.id,
                        Some(user_context.user.id),
                        shift_id,
                        Action::CLAIMED,
                        format!("User {} claimed shift {}", user_id, shift_id),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log shift claim activity: {}", e);
                }
            }

            Ok(ApiResponse::created(claim))
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
    AsyncUserContext(user_context): AsyncUserContext,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();

    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match shift_claim_repo.get_claims_by_shift(shift_id).await {
        Ok(claims) => Ok(ApiResponse::success(claims)),
        Err(err) => {
            log::error!("Error fetching claims for shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch claims")))
        }
    }
}

// Get user's own claims
pub async fn get_my_claims(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();

    match shift_claim_repo.get_claims_by_user(user_id).await {
        Ok(claims) => Ok(ApiResponse::success(claims)),
        Err(err) => {
            log::error!("Error fetching claims for user {}: {}", user_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch claims")))
        }
    }
}

// Approve a shift claim (managers/admins only)
pub async fn approve_shift_claim(
    AsyncUserContext(user_context): AsyncUserContext,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    shift_repo: web::Data<ShiftRepository>,
    path: web::Path<Uuid>,
    approval_data: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    let claim_id = path.into_inner();
    let approver_id = user_context.user_id();

    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
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
            // Assign the shift to the user
            match shift_repo.assign_shift(claim.shift_id, claim.user_id).await {
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
                    Ok(ApiResponse::success(serde_json::json!({
                        "claim": approved_claim,
                        "shift": assigned_shift
                    })))
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
    AsyncUserContext(user_context): AsyncUserContext,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    path: web::Path<Uuid>,
    rejection_data: web::Json<ApprovalRequest>,
) -> Result<HttpResponse> {
    let claim_id = path.into_inner();
    let approver_id = user_context.user_id();

    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
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
            Ok(ApiResponse::success(rejected_claim))
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
    AsyncUserContext(user_context): AsyncUserContext,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let claim_id = path.into_inner();
    let user_id = user_context.user_id();

    match shift_claim_repo.cancel_claim(claim_id, user_id).await {
        Ok(Some(cancelled_claim)) => {
            log::info!("User {} cancelled claim {}", user_id, claim_id);
            Ok(ApiResponse::success(cancelled_claim))
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
    AsyncUserContext(user_context): AsyncUserContext,
    shift_claim_repo: web::Data<ShiftClaimRepository>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match shift_claim_repo
        .get_pending_claims_by_company(user_context.company_id().unwrap_or_default())
        .await
    {
        Ok(claims) => Ok(ApiResponse::success(claims)),
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

#[derive(Debug, Deserialize)]
pub struct AssignmentResponseRequest {
    pub response: String, // "accepted" or "declined"
    pub notes: Option<String>,
}
