use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::{
    Action, AssignmentResponse, Shift, ShiftAssignment, ShiftAssignmentInput, ShiftClaimInput,
    ShiftInput, ShiftQuery, ShiftQueryType, ShiftStatus,
};
use crate::database::repositories::{
    schedule as schedule_repo, shift as shift_repo, shift_claim as shift_claim_repo,
};
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::{activity_logger, user_context::extract_context};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignShiftRequest {
    pub user_id: Uuid,
    pub acceptance_deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftAssignResponse {
    pub shift: Shift,
    pub assignment: ShiftAssignment,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectAssignShiftRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftStatusRequest {
    pub status: ShiftStatus,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignmentResponseRequest {
    pub response: AssignmentResponse, // "accepted" or "declined"
    pub notes: Option<String>,
}

// Shift handlers
pub async fn create_shift(input: web::Json<ShiftInput>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    user_context.requires_same_company(input.company_id)?;

    let shift_input = input.into_inner();

    let shift = shift_repo::create_shift(shift_input).await.map_err(|e| {
        log::error!("Failed to create shift: {}", e);
        AppError::DatabaseError(e)
    })?;

    // Log shift creation activity
    let metadata = activity_logger::metadata(vec![
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
    if let Err(e) = activity_logger::log_shift_activity(
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

// In shifts.rs handler
pub async fn get_shifts(
    query: web::Query<ShiftQuery>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_context = extract_context(&req).await?;

    match &query.query_type {
        ShiftQueryType::User(user_id) => user_context.requires_same_user(*user_id)?,
        _ => user_context.requires_manager()?,
    }

    let shifts = shift_repo::find_by_query(query.into_inner())
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shifts: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(shifts))
}

pub async fn get_shift(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let company_id = user_context.strict_company_id()?;
    let shift_id = path.into_inner();

    let shift = shift_repo::find_by_id(shift_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    user_context.requires_same_company(shift.company_id)?;

    Ok(ApiResponse::success(shift))
}

pub async fn update_shift(
    path: web::Path<Uuid>,
    input: web::Json<ShiftInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    // Check if user is admin or manager
    user_context.requires_manager()?;

    // Ensure the user has access to the company
    user_context.requires_same_company(input.company_id)?;

    let shift_id = path.into_inner();
    let company_id = user_context.strict_company_id()?;

    let shift = shift_repo::find_by_id(shift_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    // Ensure the shift belongs to the user's company
    if shift.company_id != user_context.company_id().unwrap_or_default() {
        return Err(AppError::Forbidden("You do not have access to this shift".to_string()).into());
    }

    let updated_shift = shift_repo::update_shift(shift_id, input.into_inner())
        .await
        .map_err(|e| {
            log::error!("Failed to update shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    // Log shift update activity
    let metadata = activity_logger::metadata(vec![
        ("location_id", updated_shift.location_id.to_string()),
        (
            "team_id",
            updated_shift
                .team_id
                .map_or("None".to_string(), |id| id.to_string()),
        ),
        ("start_time", updated_shift.start_time.to_string()),
        ("end_time", updated_shift.end_time.to_string()),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        user_context.company_id().unwrap_or_default(),
        Some(user_context.user.id),
        updated_shift.id,
        Action::UPDATED,
        "Shift updated".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log shift update activity: {}", e);
    }

    Ok(ApiResponse::success(updated_shift))
}

pub async fn assign_shift(
    path: web::Path<Uuid>,
    input: web::Json<AssignShiftRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;

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

    let assignment = schedule_repo::create_shift_assignment(user_id, assignment_input)
        .await
        .map_err(|e| {
            log::error!("Failed to create shift assignment: {}", e);
            AppError::DatabaseError(e)
        })?;

    let shift = shift_repo::assign_shift(shift_id, assigned_user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to assign shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    // Log shift assignment activity
    let metadata = activity_logger::metadata(vec![
        (&"assigned_user_id", assigned_user_id.to_string()),
        (&"shift_id", shift_id.to_string()),
        (&"assignment_id", assignment.id.to_string()),
        (&"location_id", shift.location_id.to_string()),
        ("start_time", shift.start_time.to_string()),
        ("end_time", shift.end_time.to_string()),
        (
            "team_id",
            shift
                .team_id
                .map_or("None".to_string(), |id| id.to_string()),
        ),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
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

    Ok(ApiResponse::success(ShiftAssignResponse {
        shift,
        assignment,
    }))
}

pub async fn unassign_shift(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;

    let shift_id = path.into_inner();
    let user_id = user_context.user_id();

    let shift = shift_repo::unassign_shift(shift_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    let metadata = activity_logger::metadata(vec![
        (&"location_id", shift.location_id.to_string()),
        (&"shift_id", shift_id.to_string()),
        ("start_time", shift.start_time.to_string()),
        ("end_time", shift.end_time.to_string()),
        (
            "team_id",
            shift
                .team_id
                .map_or("None".to_string(), |id| id.to_string()),
        ),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
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

    Ok(ApiResponse::success(shift))
}

pub async fn update_shift_status(
    path: web::Path<Uuid>,
    input: web::Json<UpdateShiftStatusRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;

    let shift_id = path.into_inner();

    let status = input.status.clone();

    let shift = shift_repo::update_shift_status(shift_id, status.clone())
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    // Log shift status update activity
    let metadata = activity_logger::metadata(vec![
        ("status", status.to_string()),
        ("shift_id", shift_id.to_string()),
        ("location_id", shift.location_id.to_string()),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
        Some(user_context.user.id),
        shift_id,
        Action::UPDATED,
        format!("Shift status updated to {}", status),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log shift status update activity: {}", e);
    };

    Ok(ApiResponse::success(shift))
}

pub async fn delete_shift(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;

    let shift_id = path.into_inner();

    shift_repo::delete_shift(shift_id)
        .await
        .map_err(|e| {
            log::error!("Failed to delete shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    // Log shift deletion activity
    let metadata = activity_logger::metadata(vec![("shift_id", shift_id.to_string())]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
        Some(user_context.user.id),
        shift_id,
        Action::DELETED,
        "Shift deleted".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log shift deletion activity: {}", e);
    }

    Ok(ApiResponse::success_message("Shift deleted successfully"))
}

// Get shift assignments for a specific shift (managers/admins only)
pub async fn get_shift_assignments(
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let shift_id = path.into_inner();

    let assignments = schedule_repo::get_shift_assignments_by_shift(shift_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch assignments for shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(assignments))
}

// Get user's pending assignments
pub async fn get_my_pending_assignments(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();

    let assignments = schedule_repo::get_pending_assignments_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to fetch pending assignments for user {}: {}",
                user_id,
                e
            );
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(assignments))
}

// Respond to a shift assignment
pub async fn respond_to_assignment(
    path: web::Path<Uuid>,
    input: web::Json<AssignmentResponseRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let assignment_id = path.into_inner();

    // Parse response
    let response = input.response.clone();

    let is_accepted = matches!(response, AssignmentResponse::Accept);

    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch assignment {}: {}", assignment_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Assignment not found".to_string()))?;

    user_context.requires_same_user(assignment.user_id)?;

    let assignment_response =
        schedule_repo::respond_to_assignment(assignment_id, response, input.notes.clone())
            .await
            .map_err(|e| {
                log::error!("Failed to respond to assignment {}: {}", assignment_id, e);
                AppError::DatabaseError(e)
            })?
            .ok_or_else(|| AppError::NotFound("Assignment not found".to_string()))?;

    // If accepted, update shift status
    if is_accepted {
        shift_repo::update_shift_status(assignment.shift_id, ShiftStatus::Assigned)
            .await
            .map_err(|e| {
                log::warn!("Failed to update shift status after acceptance: {}", e);
                AppError::DatabaseError(e)
            })?;
    }

    Ok(ApiResponse::success(assignment_response))
}

// Employee shift claiming with proper validation and workflow
pub async fn claim_shift(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let company_id = user_context.strict_company_id()?;
    let shift_id = path.into_inner();
    let user_id = user_context.user_id();

    // Get shift information for validation
    let shift_info = shift_repo::find_by_id(shift_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    // Validate shift is claimable
    if !matches!(shift_info.status, ShiftStatus::Open) {
        return Err(AppError::BadRequest("Shift is not open for claiming".to_string()).into());
    }

    // Check if shift is too close to start time (must be at least 2 hours in advance)
    let now = Utc::now();
    let time_until_shift = shift_info.start_time - now;
    if time_until_shift.num_hours() < 2 {
        return Err(AppError::BadRequest(
            "Shift must be at least 2 hours in advance to claim".to_string(),
        )
        .into());
    }

    // Check if user has already claimed this shift
    if shift_claim_repo::has_user_claimed_shift(shift_id, user_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to check if user {} has claimed shift {}: {}",
                user_id,
                shift_id,
                e
            );
            AppError::DatabaseError(e)
        })
        .is_ok()
    {
        return Err(AppError::BadRequest("You have already claimed this shift".to_string()).into());
    }

    if let Some(_team_id) = shift_info.team_id {
        // Check if user is a team member (if shift has a team)
        shift_claim_repo::is_user_team_member(shift_id, user_id)
            .await
            .map_err(|e| {
                log::error!(
                    "Failed to check team membership for shift {} user {}: {}",
                    shift_id,
                    user_id,
                    e
                );
                AppError::DatabaseError(e)
            })?
            .ok_or_else(|| {
                AppError::Forbidden("You are not a member of this shift's team".to_string())
            })?;
    }

    // Create the shift claim
    let claim_input = ShiftClaimInput { shift_id, user_id };

    let claim = shift_claim_repo::create_claim(&claim_input)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to create claim for shift {} user {}: {}",
                shift_id,
                user_id,
                e
            );
            AppError::DatabaseError(e)
        })?;

    log::info!(
        "User {} claimed shift {} - claim ID: {}",
        user_id,
        shift_id,
        claim.id
    );

    let company_id = user_context.strict_company_id()?;

    // Log shift claim activity
    let metadata = activity_logger::metadata(vec![
        ("shift_id", shift_id.to_string()),
        ("claiming_user_id", user_id.to_string()),
        ("start_time", shift_info.start_time.to_string()),
        (
            "team_id",
            shift_info
                .team_id
                .map_or("None".to_string(), |id| id.to_string()),
        ),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
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

    Ok(ApiResponse::created(claim))
}

// Get claims for a specific shift (managers/admins only)
pub async fn get_shift_claims(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let shift_id = path.into_inner();

    let claims = shift_claim_repo::get_claims_by_shift(shift_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch claims for shift {}: {}", shift_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(claims))
}

// Get user's own claims
pub async fn get_my_claims(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();

    let claims = shift_claim_repo::get_claims_by_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch claims for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(claims))
}

// Approve a shift claim (managers/admins only)
pub async fn approve_shift_claim(
    path: web::Path<Uuid>,
    approval_data: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let claim_id = path.into_inner();
    let approver_id = user_context.user_id();
    let company_id = user_context.strict_company_id()?;

    // Get the claim to approve
    let claim = shift_claim_repo::get_claim_by_id(claim_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch claim {}: {}", claim_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Claim not found".to_string()))?;

    // Approve the claim
    let approved_claim =
        shift_claim_repo::approve_claim(claim_id, approver_id, approval_data.notes.clone())
            .await
            .map_err(|e| {
                log::error!("Failed to approve claim {}: {}", claim_id, e);
                AppError::DatabaseError(e)
            })?
            .ok_or_else(|| {
                AppError::NotFound("Claim not found or already processed".to_string())
            })?;

    // Assign the shift to the user
    let assigned_shift = shift_repo::assign_shift(claim.shift_id, claim.user_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to assign shift {} after approving claim {}: {}",
                claim.shift_id,
                claim_id,
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    // Cancel any other pending claims for this shift
    shift_claim_repo::cancel_pending_claims_for_shift(claim.shift_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to cancel pending claims for shift {}: {}",
                claim.shift_id,
                e
            );
            AppError::DatabaseError(e)
        })?;

    // Log the approval activity
    let metadata = activity_logger::metadata(vec![
        ("claim_id", approved_claim.id.to_string()),
        ("shift_id", approved_claim.shift_id.to_string()),
        ("approver_id", approver_id.to_string()),
        ("user_id", approved_claim.user_id.to_string()),
        ("start_time", assigned_shift.start_time.to_string()),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
        Some(approver_id),
        approved_claim.shift_id,
        Action::APPROVED,
        format!("Claim {} approved by user {}", claim_id, approver_id),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log shift claim approval activity: {}", e);
    }

    Ok(ApiResponse::success(serde_json::json!({
        "claim": approved_claim,
        "shift": assigned_shift
    })))
}

// Reject a shift claim (managers/admins only)
pub async fn reject_shift_claim(
    path: web::Path<Uuid>,
    rejection_data: web::Json<ApprovalRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let claim_id = path.into_inner();
    let approver_id = user_context.user_id();
    let company_id = user_context.strict_company_id()?;

    let rejected_claim =
        shift_claim_repo::reject_claim(claim_id, approver_id, rejection_data.notes.clone())
            .await
            .map_err(|e| {
                log::error!("Failed to reject claim {}: {}", claim_id, e);
                AppError::DatabaseError(e)
            })?
            .ok_or_else(|| {
                AppError::NotFound("Claim not found or already processed".to_string())
            })?;

    // Log the rejection activity
    let metadata = activity_logger::metadata(vec![
        ("claim_id", rejected_claim.id.to_string()),
        ("shift_id", rejected_claim.shift_id.to_string()),
        ("approver_id", approver_id.to_string()),
        ("user_id", rejected_claim.user_id.to_string()),
        ("notes", rejection_data.notes.clone().unwrap_or_default()),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
        Some(approver_id),
        rejected_claim.shift_id,
        Action::REJECTED,
        format!("Claim {} rejected by user {}", claim_id, approver_id),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log shift claim rejection activity: {}", e);
    }

    Ok(ApiResponse::success(rejected_claim))
}

// Cancel a shift claim (by the user who made it)
pub async fn cancel_shift_claim(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let claim_id = path.into_inner();
    let user_id = user_context.user_id();
    let company_id = user_context.strict_company_id()?;

    let claim = shift_claim_repo::get_claim_by_id(claim_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch claim {}: {}", claim_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Claim not found".to_string()))?;

    // Ensure the claim belongs to the user
    if claim.user_id != user_id {
        return Err(AppError::Forbidden(
            "You do not have permission to cancel this claim".to_string(),
        )
        .into());
    }

    let cancelled_claim = shift_claim_repo::cancel_claim(claim_id, user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to cancel claim {}: {}", claim_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound("Claim not found or not cancellable".to_string()))?;

    // Log the cancellation activity
    let metadata = activity_logger::metadata(vec![
        ("claim_id", cancelled_claim.id.to_string()),
        ("shift_id", cancelled_claim.shift_id.to_string()),
        ("user_id", user_id.to_string()),
    ]);

    if let Err(e) = activity_logger::log_shift_activity(
        company_id,
        Some(user_id),
        cancelled_claim.shift_id,
        Action::CANCELLED,
        format!("Claim {} cancelled by user {}", claim_id, user_id),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log shift claim cancellation activity: {}", e);
    }

    Ok(ApiResponse::success(cancelled_claim))
}

// Get pending claims for approval (managers/admins only)
pub async fn get_pending_claims(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let claims = shift_claim_repo::get_pending_claims_by_company(
        user_context.company_id().unwrap_or_default(),
    )
    .await
    .map_err(|e| {
        log::error!("Failed to fetch pending claims: {}", e);
        AppError::DatabaseError(e)
    })?;

    Ok(ApiResponse::success(claims))
}
