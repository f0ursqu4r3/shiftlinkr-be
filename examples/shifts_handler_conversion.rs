// Example conversion of shifts handler to use UserContextService

use crate::database::repositories::company::CompanyRepository;
use crate::database::repositories::shift::ShiftRepository;
use crate::extract_user_context;
use crate::services::{UserContext, UserContextService};
use actix_web::{web, HttpRequest, HttpResponse, Result};

// BEFORE: Using Claims directly
pub async fn get_shifts_old(
    claims: Claims,
    shift_repo: web::Data<ShiftRepository>,
    company_repo: web::Data<CompanyRepository>,
    query: web::Query<ShiftQuery>,
) -> Result<HttpResponse> {
    // Manual permission checking
    let has_manager_permissions = if let Some(company_id) = query.company_id {
        company_repo
            .check_company_permission(claims.user_id(), company_id, CompanyRole::Manager)
            .await
            .unwrap_or(false)
    } else {
        false
    };

    let shifts = if let Some(user_id) = query.user_id {
        // Users can only see their own shifts unless they are admin/manager
        if !has_manager_permissions && user_id != claims.user_id() {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        shift_repo.get_shifts_by_user(user_id).await
    } else if let Some(location_id) = query.location_id {
        shift_repo.get_shifts_by_location(location_id).await
    } else {
        // For general queries, only admin/manager can see all shifts
        if !has_manager_permissions {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        shift_repo.get_all_shifts().await
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

// AFTER: Using UserContextService
pub async fn get_shifts_new(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
    shift_repo: web::Data<ShiftRepository>,
    query: web::Query<ShiftQuery>,
) -> Result<HttpResponse> {
    // Extract user context (includes user, company, and role info)
    let user_context = extract_user_context!(user_context_service, &req);

    let shifts = if let Some(user_id) = query.user_id {
        // Check if user can access this user's shifts
        if !user_context.can_access_user_resource(user_id) {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Insufficient permissions")));
        }
        shift_repo.get_shifts_by_user(user_id).await
    } else if let Some(location_id) = query.location_id {
        // Location-based queries might require company membership
        if let Some(company_id) = user_context.company_id() {
            shift_repo
                .get_shifts_by_location_and_company(location_id, company_id)
                .await
        } else {
            shift_repo.get_shifts_by_location(location_id).await
        }
    } else {
        // General queries require manager or admin permissions
        if !user_context.is_manager_or_admin() {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Manager or admin access required")));
        }

        // Get shifts for user's company if they have one
        if let Some(company_id) = user_context.company_id() {
            shift_repo.get_shifts_by_company(company_id).await
        } else {
            shift_repo.get_all_shifts().await
        }
    };

    match shifts {
        Ok(shifts) => Ok(HttpResponse::Ok().json(ApiResponse::success(shifts))),
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

// Example: Create shift with automatic company context
pub async fn create_shift_new(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
    input: web::Json<ShiftInput>,
    shift_repo: web::Data<ShiftRepository>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);

    // Only admins and managers can create shifts
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Manager or admin access required to create shifts"
        })));
    }

    // Ensure user has a company
    let company_id = user_context
        .company_id()
        .ok_or_else(|| actix_web::error::ErrorBadRequest("User must belong to a company"))?;

    let mut shift_input = input.into_inner();

    // Automatically set the company context if not provided
    // This prevents users from creating shifts for other companies
    if shift_input.company_id.is_none() {
        shift_input.company_id = Some(company_id);
    } else if shift_input.company_id != Some(company_id) && !user_context.is_admin() {
        // Non-admins cannot create shifts for other companies
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Cannot create shifts for other companies"
        })));
    }

    match shift_repo.create_shift(shift_input).await {
        Ok(shift) => {
            log::info!(
                "Shift created by user {} for company {}",
                user_context.user_id(),
                company_id
            );
            Ok(HttpResponse::Created().json(shift))
        }
        Err(e) => {
            log::error!("Failed to create shift: {}", e);
            Ok(HttpResponse::BadRequest().json(json!({
                "error": e.to_string()
            })))
        }
    }
}

// Example: User-specific operations with automatic ownership check
pub async fn get_my_shifts(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
    shift_repo: web::Data<ShiftRepository>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);

    // Get shifts for the current user
    match shift_repo.get_shifts_by_user(user_context.user_id()).await {
        Ok(shifts) => Ok(HttpResponse::Ok().json(json!({
            "shifts": shifts,
            "user": {
                "id": user_context.user_id(),
                "email": user_context.user_email(),
                "company": user_context.company_name(),
                "role": user_context.role(),
            }
        }))),
        Err(err) => {
            log::error!(
                "Error fetching shifts for user {}: {}",
                user_context.user_id(),
                err
            );
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch your shifts"
            })))
        }
    }
}

// Example: Admin operation with automatic permission checking
pub async fn delete_shift_admin(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
    path: web::Path<Uuid>,
    shift_repo: web::Data<ShiftRepository>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    let shift_id = path.into_inner();

    // Only admins can delete shifts
    if !user_context.is_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Admin access required to delete shifts"
        })));
    }

    // Ensure admin is deleting shifts from their own company (unless super admin)
    if let Some(company_id) = user_context.company_id() {
        // Check if shift belongs to user's company
        if let Ok(Some(shift)) = shift_repo.get_shift_by_id(shift_id).await {
            if shift.company_id != company_id {
                return Ok(HttpResponse::Forbidden().json(json!({
                    "error": "Cannot delete shifts from other companies"
                })));
            }
        }
    }

    match shift_repo.delete_shift(shift_id).await {
        Ok(_) => {
            log::info!(
                "Shift {} deleted by admin {}",
                shift_id,
                user_context.user_id()
            );
            Ok(HttpResponse::Ok().json(json!({
                "message": "Shift deleted successfully"
            })))
        }
        Err(err) => {
            log::error!("Error deleting shift {}: {}", shift_id, err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete shift"
            })))
        }
    }
}
