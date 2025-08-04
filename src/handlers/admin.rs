use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::{
    Action, CompanyRole, CreateUpdateLocationInput, LocationInput, TeamInput,
};
use crate::database::repositories::company::CompanyRepository;
use crate::database::repositories::location::LocationRepository;
use crate::database::repositories::user::UserRepository;
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::repositories::TeamRepository;
use crate::services::user_context::AsyncUserContext;
use crate::services::ActivityLogger;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub name: String,
    pub email: String,
    pub role: Option<CompanyRole>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: CompanyRole,
    pub company_id: Uuid,
    pub hire_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// Location handlers
pub async fn create_location(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    activity_logger: web::Data<ActivityLogger>,
    input: web::Json<CreateUpdateLocationInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();

    user_context.requires_manager()?;

    let company_id = user_context
        .company_id()
        .ok_or_else(|| AppError::BadRequest("User does not belong to any company".to_string()))?;

    let location_input = input.into_inner();

    let location_name = location_input.name.clone();

    let location = location_repo
        .create_location(LocationInput {
            name: location_input.name,
            address: location_input.address,
            phone: location_input.phone,
            email: location_input.email,
            company_id,
        })
        .await
        .map_err(|err| {
            log::error!("Error creating location: {}", err);
            AppError::DatabaseError(err)
        })?;

    // Log location creation activity
    let metadata = ActivityLogger::metadata(vec![
        ("location_name", location_name),
        ("location_id", location.id.to_string()),
    ]);

    if let Err(e) = activity_logger
        .log_location_activity(
            company_id,
            Some(user_id),
            location.id,
            Action::CREATED,
            format!("Location '{}' created", location.name),
            Some(metadata),
            &req,
        )
        .await
    {
        log::warn!("Failed to log location creation activity: {}", e);
    }

    Ok(ApiResponse::success(location))
}

pub async fn get_locations(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    company_repo: web::Data<CompanyRepository>,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();

    let companies = company_repo
        .get_companies_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching user companies: {}", e);
            AppError::DatabaseError(e)
        })?;

    if companies.is_empty() {
        return Ok(ApiResponse::success(Vec::<LocationInput>::new()));
    }

    let company_ids: Vec<Uuid> = companies.iter().map(|c| c.id).collect();

    let all_locations = location_repo
        .get_locations_by_company_ids(company_ids)
        .await
        .map_err(|e| {
            log::error!("Error fetching locations: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(all_locations))
}

pub async fn get_location(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let location_id = path.into_inner();
    let user_id = user_context.user_id();

    let location = location_repo
        .find_by_id(location_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location {}: {}", location_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location {} not found", location_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    // Check if user has access to this location's company
    company_repo
        .check_user_company_access(user_id, location.company_id)
        .await
        .map_err(|e| {
            log::error!(
                "Error checking company access for user {} and location {}: {}",
                user_id,
                location_id,
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!(
                "User {} does not have access to location {}",
                user_id,
                location_id
            );
            AppError::PermissionDenied("Insufficient permissions to access location".to_string())
        })?;

    Ok(ApiResponse::success(location))
}

pub async fn update_location(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
    input: web::Json<CreateUpdateLocationInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    let company_id = user_context
        .company_id()
        .ok_or_else(|| AppError::BadRequest("User does not belong to any company".to_string()))?;

    user_context.requires_manager()?;

    let location_id = path.into_inner();

    let location = location_repo
        .update_location(
            location_id,
            LocationInput {
                name: input.name.clone(),
                address: input.address.clone(),
                phone: input.phone.clone(),
                email: input.email.clone(),
                company_id,
            },
        )
        .await
        .map_err(|err| {
            log::error!("Error updating location {}: {}", location_id, err);
            AppError::DatabaseError(err)
        })?
        .ok_or_else(|| {
            log::warn!("Location {} not found", location_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    Ok(ApiResponse::success(location))
}

pub async fn delete_location(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let location_id = path.into_inner();

    // Get the location to check permissions
    let location = location_repo
        .find_by_id(location_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location {}: {}", location_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location {} not found", location_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    user_context.requires_same_company(location.company_id)?;

    // Proceed with deletion
    location_repo
        .delete_location(location_id)
        .await
        .map_err(|e| {
            log::error!("Error deleting location {}: {}", location_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location {} not found for deletion", location_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    Ok(ApiResponse::success_message(
        "Location deleted successfully",
    ))
}

// Team handlers
pub async fn create_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    activity_logger: web::Data<ActivityLogger>,
    input: web::Json<TeamInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Get the location to determine which company it belongs to
    let location_id = input.location_id;
    let team_input = input.into_inner();
    let team_name = team_input.name.clone();
    let user_id = user_context.user_id();

    let location = location_repo
        .find_by_id(location_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location {}: {}", location_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location {} not found", location_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    user_context.requires_same_company(location.company_id)?;

    let team = team_repo.create_team(team_input).await.map_err(|e| {
        log::error!("Error creating team: {}", e);
        AppError::DatabaseError(e)
    })?;

    let metadata = ActivityLogger::metadata(vec![
        ("team_name", team_name),
        ("team_id", team.id.to_string()),
        ("location_id", location_id.to_string()),
    ]);

    if let Err(e) = activity_logger
        .log_team_activity(
            location.company_id,
            Some(user_id),
            team.id,
            Action::CREATED,
            format!("Team '{}' created in location {}", team.name, location_id),
            Some(metadata),
            &req,
        )
        .await
    {
        log::warn!("Failed to log team creation activity: {}", e);
    }

    Ok(ApiResponse::success(team))
}

pub async fn get_teams(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    query: web::Query<TeamQuery>,
) -> Result<HttpResponse> {
    let teams = if let Some(location_id) = query.location_id {
        let location = location_repo
            .find_by_id(location_id)
            .await
            .map_err(|e| {
                log::error!("Error fetching location {}: {}", location_id, e);
                AppError::DatabaseError(e)
            })?
            .ok_or_else(|| {
                log::warn!("Location {} not found", location_id);
                AppError::NotFound("Location not found".to_string())
            })?;

        user_context.requires_same_company(location.company_id)?;

        team_repo
            .get_teams_by_location(location_id)
            .await
            .map_err(|e| {
                log::error!("Error fetching teams for location {}: {}", location_id, e);
                AppError::DatabaseError(e)
            })?
    } else {
        user_context.requires_manager()?;
        let company_id = user_context.strict_company_id()?;

        team_repo
            .get_all_teams_for_company(company_id)
            .await
            .map_err(|e| {
                log::error!("Error fetching teams for company: {}", e);
                AppError::DatabaseError(e)
            })?
    };

    Ok(ApiResponse::success(teams))
}

pub async fn get_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    user_context.requires_manager()?;

    let location = get_location_for_team(&location_repo, team_id).await?;

    user_context.requires_same_company(location.company_id)?;

    let team = team_repo
        .get_team_by_id(team_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching team {}: {}", team_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Team {} not found", team_id);
            AppError::NotFound("Team not found".to_string())
        })?;

    Ok(ApiResponse::success(team))
}

pub async fn update_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    path: web::Path<Uuid>,
    input: web::Json<TeamInput>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    user_context.requires_manager()?;

    let location = location_repo
        .find_by_team_id(team_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location by team {}: {}", team_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location for team {} not found", team_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    user_context.requires_same_company(location.company_id)?;

    let team = team_repo
        .update_team(team_id, input.into_inner())
        .await
        .map_err(|e| {
            log::error!("Error fetching team {}: {}", team_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Team {} not found", team_id);
            AppError::NotFound("Team not found".to_string())
        })?;

    Ok(ApiResponse::success(team))
}

pub async fn delete_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    user_context.requires_manager()?;

    let location = get_location_for_team(&location_repo, team_id).await?;

    user_context.requires_same_company(location.company_id)?;

    team_repo
        .delete_team(team_id)
        .await
        .map_err(|e| {
            log::error!("Error deleting team {}: {}", team_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Team {} not found for deletion", team_id);
            AppError::NotFound("Team not found".to_string())
        })?;

    Ok(ApiResponse::success_message("Team deleted successfully"))
}

// Team member handlers
pub async fn add_team_member(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse> {
    let (team_id, user_id) = path.into_inner();

    user_context.requires_manager()?;

    let location = get_location_for_team(&location_repo, team_id).await?;

    user_context.requires_same_company(location.company_id)?;

    let team_member = team_repo
        .add_team_member(team_id, user_id)
        .await
        .map_err(|e| {
            log::error!("Error adding team member to team {}: {}", team_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(team_member))
}

pub async fn get_team_members(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    user_context.requires_manager()?;

    let location = get_location_for_team(&location_repo, team_id).await?;

    user_context.requires_same_company(location.company_id)?;

    let members = team_repo.get_team_members(team_id).await.map_err(|e| {
        log::error!("Error fetching team members for team {}: {}", team_id, e);
        AppError::DatabaseError(e)
    })?;

    Ok(ApiResponse::success(members))
}

pub async fn remove_team_member(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    team_repo: web::Data<TeamRepository>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse> {
    let (team_id, user_id) = path.into_inner();

    user_context.requires_manager()?;

    let location = get_location_for_team(&location_repo, team_id).await?;

    user_context.requires_same_company(location.company_id)?;

    team_repo
        .remove_team_member(team_id, user_id)
        .await
        .map_err(|e| {
            log::error!("Error removing team member: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Team member not found in team {}", team_id);
            AppError::NotFound("Team member not found".to_string())
        })?;

    Ok(ApiResponse::success_message(
        "Team member removed successfully",
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamQuery {
    pub location_id: Option<Uuid>,
}

// User management handlers
pub async fn get_users(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repo: web::Data<CompanyRepository>,
) -> Result<HttpResponse> {
    user_context.requires_manager()?;

    let company_id = user_context.company_id().ok_or_else(|| {
        return AppError::BadRequest("User does not belong to any company".to_string());
    })?;

    let employees: Vec<UserResponse> = company_repo
        .get_company_employees(company_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching company employees: {}", e);
            AppError::DatabaseError(e)
        })?
        .into_iter()
        .map(|employee| UserResponse {
            id: employee.id,
            email: employee.email,
            name: employee.name,
            role: employee.role,
            company_id,
            hire_date: employee.hire_date.map(|d| d.format("%Y-%m-%d").to_string()),
            created_at: employee
                .created_at
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            updated_at: employee
                .updated_at
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        })
        .collect();

    Ok(ApiResponse::success(employees))
}

pub async fn update_user(
    AsyncUserContext(user_context): AsyncUserContext,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<String>,
    input: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id_to_update = path
        .into_inner()
        .parse::<Uuid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let user_id = user_context.user_id();
    let update_request = input.into_inner();

    // Check permissions based on whether role is being updated
    if update_request.role.is_some() {
        user_context.requires_manager()?;
    } else {
        user_context.requires_same_user(user_id_to_update)?;
    }

    // Get the company ID from user context
    let company_id = user_context
        .company_id()
        .ok_or_else(|| AppError::BadRequest("User does not belong to any company".to_string()))?;

    // Check if the user to update belongs to requesting user's company
    let user_company_info = company_repo
        .find_user_company_info_by_id(user_id_to_update, company_id)
        .await?
        .ok_or_else(|| {
            AppError::PermissionDenied("User does not belong to the same company".to_string())
        })?;

    // Update basic user information
    let updated_user = user_repo
        .update_user(
            user_id_to_update,
            &update_request.name,
            &update_request.email,
        )
        .await?;

    let final_role = if let Some(new_role) = update_request.role {
        // If the user is trying to change their own role, ensure they are not demoting themselves from admin
        if user_id == user_id_to_update
            && user_company_info.role == CompanyRole::Admin
            && new_role != CompanyRole::Admin
        {
            return Err(AppError::PermissionDenied(
                "You cannot demote yourself from admin role".to_string(),
            ));
        }

        // Update the user's role in the company
        company_repo
            .update_employee_role(company_id, user_id_to_update, &new_role)
            .await?;

        new_role
    } else {
        user_company_info.role
    };

    Ok(ApiResponse::success(UserResponse {
        id: updated_user.id,
        email: updated_user.email,
        name: updated_user.name,
        role: final_role,
        company_id,
        hire_date: user_company_info
            .hire_date
            .map(|d| d.format("%Y-%m-%d").to_string()),
        created_at: updated_user
            .created_at
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string(),
        updated_at: updated_user
            .updated_at
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string(),
    }))
}

pub async fn delete_user(
    AsyncUserContext(user_context): AsyncUserContext,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id_to_delete = path.into_inner();

    // Only admins can delete users - check company-specific admin role
    user_context.requires_manager()?;

    // Get the company ID from user context
    let company_id = match user_context.company_id() {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::BadRequest()
                .json(ApiResponse::error("User does not belong to any company")));
        }
    };

    // Check if user is admin in the specific company
    match company_repo.get_company_employees(company_id).await {
        Ok(employees) => {
            // Check if the user to delete is an admin
            let admin_count = employees
                .iter()
                .filter(|e| e.role == CompanyRole::Admin)
                .count();
            if admin_count <= 1 && employees.iter().any(|e| e.id == user_id_to_delete) {
                // If the user to delete is the only admin, prevent deletion
                return Ok(HttpResponse::BadRequest()
                    .json(ApiResponse::error("Cannot delete the only admin user")));
            }
        }
        Err(err) => {
            log::error!("Error fetching company employees: {}", err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch company employees")));
        }
    }

    match user_repo.delete_user(user_id_to_delete).await {
        Ok(()) => Ok(ApiResponse::<()>::success_message(
            "User deleted successfully",
        )),
        Err(err) => {
            log::error!("Error deleting user {}: {}", user_id_to_delete, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to delete user")))
        }
    }
}

// Utilities
async fn get_location_for_team(
    location_repo: &LocationRepository,
    team_id: Uuid,
) -> Result<crate::database::models::Location, AppError> {
    location_repo
        .find_by_team_id(team_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location by team {}: {}", team_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location for team {} not found", team_id);
            AppError::NotFound("Location not found".to_string())
        })
}
