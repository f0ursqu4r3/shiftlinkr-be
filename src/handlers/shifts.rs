use actix_web::{HttpResponse, Result, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        models::{
            Action, CreateUpdateShiftInput, Shift, ShiftAssignment,
            ShiftAssignmentInput, ShiftClaimInput, ShiftClaimResponse, ShiftQuery, ShiftQueryType,
            ShiftStatus,
        },
        repositories::{
            schedule as schedule_repo, shift as shift_repo, shift_claim as shift_claim_repo,
        },
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::request_info::RequestInfo,
    services::{activity_logger, user_context::UserContext},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignShiftInput {
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
pub struct DirectAssignShiftInput {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftStatusInput {
    pub status: ShiftStatus,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalInput {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignmentResponseInput {
    pub response: String, // "accept" or "decline"
    pub notes: Option<String>,
}

// Shift handlers
pub async fn create_shift(
    ctx: UserContext,
    input: web::Json<CreateUpdateShiftInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    ctx.requires_same_company(input.company_id)?;

    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let shift_input = input.into_inner();

    let shift = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let shift = shift_repo::create_shift(tx, shift_input).await?;

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

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                shift.id,
                &Action::CREATED,
                "Shift created".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(shift)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::created(shift))
}

// In shifts.rs handler
pub async fn get_shifts(
    ctx: UserContext,
    query: web::Query<ShiftQuery>,
) -> Result<HttpResponse, AppError> {
    match &query.query_type {
        ShiftQueryType::User(user_id) => ctx.requires_same_user(*user_id)?,
        _ => ctx.requires_manager()?,
    }

    let shifts = shift_repo::find_by_query(query.into_inner())
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shifts: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(shifts))
}

pub async fn get_shift(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let company_id = ctx.strict_company_id()?;
    let shift_id = path.into_inner();

    let shift = shift_repo::find_by_id(shift_id, company_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    ctx.requires_same_company(shift.company_id)?;

    Ok(ApiResponse::success(shift))
}

pub async fn update_shift(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<CreateUpdateShiftInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    ctx.requires_manager()?;

    // Ensure the user has access to the company
    ctx.requires_same_company(input.company_id)?;

    let shift_id = path.into_inner();
    let company_id = ctx.strict_company_id()?;
    let user_id = ctx.user_id();

    let updated_shift = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let shift = shift_repo::find_by_id(shift_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            // Ensure the shift belongs to the user's company
            if shift.company_id != company_id {
                return Err(AppError::Forbidden(
                    "You do not have access to this shift".to_string(),
                ));
            }

            let updated_shift = shift_repo::update_shift(tx, shift_id, input.into_inner())
                .await?
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

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                updated_shift.id,
                &Action::UPDATED,
                "Shift updated".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(updated_shift)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(updated_shift))
}

pub async fn assign_shift(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<AssignShiftInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;

    let shift_id = path.into_inner();
    let assigned_user_id = input.user_id;
    let acceptance_deadline = input.acceptance_deadline;

    let user_id = ctx.user_id();

    let (shift, assignment) = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Create shift assignment using schedule repository
            let assignment_input = ShiftAssignmentInput {
                shift_id,
                user_id: assigned_user_id,
                acceptance_deadline,
            };

            let assignment =
                schedule_repo::create_shift_assignment(tx, user_id, assignment_input).await?;

            let shift = shift_repo::assign_shift(tx, shift_id, assigned_user_id)
                .await?
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

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                shift_id,
                &Action::ASSIGNED,
                format!(
                    "Shift assigned to user {} via assignment system",
                    assigned_user_id
                ),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok((shift, assignment))
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(ShiftAssignResponse {
        shift,
        assignment,
    }))
}

pub async fn unassign_shift(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;

    let shift_id = path.into_inner();
    let user_id = ctx.user_id();

    let shift = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let shift = shift_repo::unassign_shift(tx, shift_id)
                .await?
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

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                shift_id,
                &Action::UNASSIGNED,
                "Shift unassigned".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(shift)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(shift))
}

pub async fn update_shift_status(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<UpdateShiftStatusInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;
    let user_id = ctx.user_id();

    let shift_id = path.into_inner();
    let status = input.status.clone();

    let shift = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let shift = shift_repo::update_shift_status(tx, shift_id, status.clone())
                .await?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            // Log shift status update activity
            let metadata = activity_logger::metadata(vec![
                ("status", status.to_string()),
                ("shift_id", shift_id.to_string()),
                ("location_id", shift.location_id.to_string()),
            ]);

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                shift_id,
                &Action::UPDATED,
                format!("Shift status updated to {}", status),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(shift)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(shift))
}

pub async fn delete_shift(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;
    let user_id = ctx.user_id();

    let shift_id = path.into_inner();

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            shift_repo::delete_shift(tx, shift_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            // Log shift deletion activity
            let metadata = activity_logger::metadata(vec![("shift_id", shift_id.to_string())]);

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                shift_id,
                &Action::DELETED,
                "Shift deleted".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success_message("Shift deleted successfully"))
}

// Get shift assignments for a specific shift (managers/admins only)
pub async fn get_shift_assignments(
    path: web::Path<Uuid>,
    ctx: UserContext,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

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
pub async fn get_my_pending_assignments(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

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
    ctx: UserContext,
    input: web::Json<AssignmentResponseInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let assignment_id = path.into_inner();

    // Parse response
    let response = input.response.clone();
    let is_accepted = response.as_str() == "accept";

    let assignment = schedule_repo::get_shift_assignment(assignment_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Assignment not found".to_string()))?;

    ctx.requires_same_user(assignment.user_id)?;
    let company_id = ctx.strict_company_id()?;

    let action = match response.as_str() {
        "accept" => Action::ACCEPTED,
        "decline" => Action::DECLINED,
        _ => Action::DECLINED, // Default case
    };

    let assignment_response = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let assignment_response = schedule_repo::respond_to_assignment(
                tx,
                assignment_id,
                response.clone(),
                input.notes.clone(),
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Assignment not found".to_string()))?;

            // If accepted, update shift status
            if is_accepted {
                shift_repo::update_shift_status(tx, assignment.shift_id, ShiftStatus::Assigned)
                    .await?;
            }

            // Log assignment response activity
            let metadata = activity_logger::metadata(vec![
                ("assignment_id", assignment_id.to_string()),
                ("response", response.to_string()),
                ("notes", input.notes.clone().unwrap_or_default()),
            ]);

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(assignment.user_id),
                assignment.shift_id,
                &action,
                format!("User responded to assignment with {}", response),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(assignment_response)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(assignment_response))
}

// Employee shift claiming with proper validation and workflow
pub async fn claim_shift(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let company_id = ctx.strict_company_id()?;
    let shift_id = path.into_inner();
    let user_id = ctx.user_id();

    let claim = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Get shift information for validation
            let shift_info = shift_repo::find_by_id(shift_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            // Validate shift is claimable
            if !matches!(shift_info.status, ShiftStatus::Open) {
                return Err(AppError::BadRequest(
                    "Shift is not open for claiming".to_string(),
                ));
            }

            // Check if shift is too close to start time (must be at least 2 hours in advance)
            let now = Utc::now();
            let time_until_shift = shift_info.start_time - now;
            if time_until_shift.num_hours() < 2 {
                return Err(AppError::BadRequest(
                    "Shift must be at least 2 hours in advance to claim".to_string(),
                ));
            }

            // Check if user has already claimed this shift
            if shift_claim_repo::has_user_claimed_shift(shift_id, user_id)
                .await
                .map_err(AppError::from)
                .is_ok()
            {
                return Err(AppError::BadRequest(
                    "You have already claimed this shift".to_string(),
                ));
            }

            if let Some(_team_id) = shift_info.team_id {
                // Check if user is a team member (if shift has a team)
                shift_claim_repo::is_user_team_member(shift_id, user_id)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| {
                        AppError::Forbidden("You are not a member of this shift's team".to_string())
                    })?;
            }

            // Create the shift claim
            let claim_input = ShiftClaimInput { shift_id, user_id };

            let claim = shift_claim_repo::create_claim(tx, &claim_input).await?;

            log::info!(
                "User {} claimed shift {} - claim ID: {}",
                user_id,
                shift_id,
                claim.id
            );

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

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                shift_id,
                &Action::CLAIMED,
                "User claimed shift".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(claim)
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::created(claim))
}

// Get claims for a specific shift (managers/admins only)
pub async fn get_shift_claims(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let shift_id = path.into_inner();

    let claims = shift_claim_repo::get_claims_by_shift(shift_id)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(claims))
}

// Get user's own claims
pub async fn get_my_claims(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let claims = shift_claim_repo::get_claims_by_user(user_id)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(claims))
}

// Approve a shift claim (managers/admins only)
pub async fn approve_shift_claim(
    path: web::Path<Uuid>,
    ctx: UserContext,
    approval_data: web::Json<ApprovalInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let claim_id = path.into_inner();
    let approver_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;

    let (claim, shift) = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Approve the claim
            let claim = shift_claim_repo::approve_claim(
                tx,
                claim_id,
                approver_id,
                approval_data.notes.clone(),
            )
            .await?
            .ok_or_else(|| {
                AppError::NotFound("Claim not found or already processed".to_string())
            })?;

            // Assign the shift to the user
            let shift = shift_repo::assign_shift(tx, claim.shift_id, claim.user_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            // Cancel any other pending claims for this shift
            shift_claim_repo::cancel_pending_claims_for_shift(tx, claim.shift_id).await?;

            // Log the approval activity
            let metadata = activity_logger::metadata(vec![
                ("claim_id", claim.id.to_string()),
                ("shift_id", claim.shift_id.to_string()),
                ("approver_id", approver_id.to_string()),
                ("user_id", claim.user_id.to_string()),
                ("start_time", shift.start_time.to_string()),
            ]);

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(approver_id),
                claim.shift_id,
                &Action::APPROVED,
                "Claim approved".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok((claim, shift))
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(ShiftClaimResponse { claim, shift }))
}

// Reject a shift claim (managers/admins only)
pub async fn reject_shift_claim(
    path: web::Path<Uuid>,
    ctx: UserContext,
    rejection_data: web::Json<ApprovalInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let claim_id = path.into_inner();
    let approver_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;

    let (claim, shift) = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let claim = shift_claim_repo::reject_claim(
                tx,
                claim_id,
                approver_id,
                rejection_data.notes.clone(),
            )
            .await?
            .ok_or_else(|| {
                AppError::NotFound("Claim not found or already processed".to_string())
            })?;

            let shift = shift_repo::assign_shift(tx, claim.shift_id, claim.user_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            // Log the rejection activity
            let metadata = activity_logger::metadata(vec![
                ("claim_id", claim.id.to_string()),
                ("shift_id", claim.shift_id.to_string()),
                ("approver_id", approver_id.to_string()),
                ("user_id", claim.user_id.to_string()),
                ("notes", rejection_data.notes.clone().unwrap_or_default()),
            ]);

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(approver_id),
                claim.shift_id,
                &Action::REJECTED,
                "Claim rejected".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok((claim, shift))
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(ShiftClaimResponse { claim, shift }))
}

// Cancel a shift claim (by the user who made it)
pub async fn cancel_shift_claim(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let claim_id = path.into_inner();
    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;

    let (claim, shift) = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let claim = shift_claim_repo::get_claim_by_id(claim_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Claim not found".to_string()))?;

            // Ensure the claim belongs to the user
            if claim.user_id != user_id {
                return Err(AppError::Forbidden(
                    "You do not have permission to cancel this claim".to_string(),
                ));
            }

            let cancelled_claim = shift_claim_repo::cancel_claim(tx, claim_id, user_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound("Claim not found or not cancellable".to_string())
                })?;

            let shift = shift_repo::assign_shift(tx, claim.shift_id, claim.user_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            // Log the cancellation activity
            let metadata = activity_logger::metadata(vec![
                ("claim_id", cancelled_claim.id.to_string()),
                ("shift_id", cancelled_claim.shift_id.to_string()),
                ("user_id", user_id.to_string()),
            ]);

            activity_logger::log_shift_activity(
                tx,
                company_id,
                Some(user_id),
                cancelled_claim.shift_id,
                &Action::CANCELLED,
                "Claim cancelled".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok((cancelled_claim, shift))
        })
    })
    .await?;

    // Invalidate cached GETs
    cache.bump();
    Ok(ApiResponse::success(ShiftClaimResponse { claim, shift }))
}

// Get pending claims for approval (managers/admins only)
pub async fn get_pending_claims(ctx: UserContext) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let claims = shift_claim_repo::get_pending_claims_by_company(ctx.strict_company_id()?)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(claims))
}
