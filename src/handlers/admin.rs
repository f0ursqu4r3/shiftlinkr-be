use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::panic::Location;
use uuid::Uuid;

use crate::database::models::{
    Action, CompanyRole, CreateUpdateLocationInput, LocationInput, TeamInput,
};
use crate::database::repositories::company::CompanyRepository;
use crate::database::repositories::location::LocationRepository;
use crate::database::repositories::user::UserRepository;
use crate::handlers::shared::ApiResponse;
use crate::services::auth::Claims;
use crate::services::user_context::AsyncUserContext;
use crate::services::ActivityLogger;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub name: String,
    pub email: String,
    pub role: Option<String>, // Company-specific role updates through company_employees table
    pub company_id: Option<Uuid>, // Required for role updates
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub company_id: Uuid,
    pub hire_date: Option<String>,
    pub is_primary: bool,
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
    // Check if user is admin or manager for the specified company
    let user_id = user_context.user_id();

    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to create location",
        )));
    }

    let company_id = match user_context.company_id() {
        Some(company_id) => company_id,
        None => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "User does not belong to any company",
            )));
        }
    };

    let location_input = input.into_inner();

    let location_name = location_input.name.clone();

    match location_repo
        .create_location(LocationInput {
            name: location_input.name,
            address: location_input.address,
            phone: location_input.phone,
            email: location_input.email,
            company_id,
        })
        .await
    {
        Ok(location) => {
            // Log location creation activity
            let mut metadata = HashMap::new();
            metadata.insert(
                "location_name".to_string(),
                serde_json::Value::String(location_name),
            );
            metadata.insert(
                "location_id".to_string(),
                serde_json::Value::String(location.id.to_string()),
            );

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

            Ok(HttpResponse::Created().json(ApiResponse::success(location)))
        }
        Err(err) => {
            log::error!("Error creating location: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to create location")))
        }
    }
}

pub async fn get_locations(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    company_repo: web::Data<CompanyRepository>,
) -> Result<HttpResponse> {
    // Get all companies the user has access to
    let user_id = user_context.user_id();

    match company_repo.get_companies_for_user(user_id).await {
        Ok(companies) => {
            let mut all_locations = Vec::new();
            // Get locations for each company the user has access to
            for company in companies {
                match location_repo.get_locations_by_company(company.id).await {
                    Ok(mut locations) => {
                        all_locations.append(&mut locations);
                    }
                    Err(err) => {
                        log::error!(
                            "Error fetching locations for company {}: {}",
                            company.id,
                            err
                        );
                        // Continue with other companies instead of failing completely
                    }
                }
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(all_locations)))
        }
        Err(err) => {
            log::error!("Error fetching user companies: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch locations")))
        }
    }
}

pub async fn get_location(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let location_id = path.into_inner();

    match location_repo.find_by_id(location_id).await {
        Ok(Some(location)) => {
            // Check if user has access to this location
            if user_context.company_id() != Some(location.company_id) {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to access location",
                )));
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(location)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found"))),
        Err(err) => {
            log::error!("Error fetching location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch location")))
        }
    }
}

pub async fn update_location(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
    input: web::Json<CreateUpdateLocationInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    let company_id = match user_context.company_id() {
        Some(company_id) => company_id,
        None => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "User does not belong to any company",
            )));
        }
    };

    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to update location",
        )));
    }

    let location_id = path.into_inner();

    match location_repo
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
    {
        Ok(Some(location)) => Ok(HttpResponse::Ok().json(ApiResponse::success(location))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found"))),
        Err(err) => {
            log::error!("Error updating location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to update location")))
        }
    }
}

pub async fn delete_location(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let location_id = path.into_inner();

    // Get the location to check permissions
    let location = match location_repo.find_by_id(location_id).await {
        Ok(Some(location)) => location,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
        }
        Err(err) => {
            log::error!("Error fetching location {}: {}", location_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch location")));
        }
    };

    // Check if user is admin
    if !user_context.is_manager_or_admin() || user_context.company_id() != Some(location.company_id)
    {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to update location",
        )));
    }

    // Proceed with deletion
    match location_repo.delete_location(location_id).await {
        Ok(true) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success("Location deleted successfully")))
        }
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found"))),
        Err(err) => {
            log::error!("Error deleting location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to delete location")))
        }
    }
}

// Team handlers
pub async fn create_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    activity_logger: web::Data<ActivityLogger>,
    input: web::Json<TeamInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Get the location to determine which company it belongs to
    let location_id = input.location_id;
    let team_input = input.into_inner();
    let team_name = team_input.name.clone();
    let user_id = user_context.user_id();

    let company_id = match location_repo.find_by_id(location_id).await {
        Ok(Some(location)) => {
            // Check if user is admin or manager for the location's company
            if !user_context.is_manager_or_admin()
                || user_context.company_id() != Some(location.company_id)
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to update location",
                )));
            }
            location.company_id
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
        }
        Err(err) => {
            log::error!("Error fetching location {}: {}", location_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to verify location")));
        }
    };

    match location_repo.create_team(team_input).await {
        Ok(team) => {
            // Log team creation activity
            let mut metadata = HashMap::new();
            metadata.insert(
                "team_name".to_string(),
                serde_json::Value::String(team_name),
            );
            metadata.insert(
                "team_id".to_string(),
                serde_json::Value::String(team.id.to_string()),
            );
            metadata.insert(
                "location_id".to_string(),
                serde_json::Value::String(location_id.to_string()),
            );

            if let Err(e) = activity_logger
                .log_team_activity(
                    company_id,
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

            Ok(HttpResponse::Created().json(ApiResponse::success(team)))
        }
        Err(err) => {
            log::error!("Error creating team: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to create team")))
        }
    }
}

pub async fn get_teams(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    query: web::Query<TeamQuery>,
) -> Result<HttpResponse> {
    let teams = if let Some(location_id) = query.location_id {
        // If location_id is provided, get the location
        // then verify the location is accessible by the user
        match location_repo.find_by_id(location_id).await {
            Ok(Some(location)) => {
                // Check if user has access to this location
                if user_context.company_id() != Some(location.company_id) {
                    return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                        "Insufficient permissions to access location",
                    )));
                }
            }
            Ok(None) => {
                return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
            }
            Err(err) => {
                log::error!("Error fetching location {}: {}", location_id, err);
                return Ok(HttpResponse::InternalServerError()
                    .json(ApiResponse::error("Failed to fetch location")));
            }
        }
        location_repo.get_teams_by_location(location_id).await
    } else if user_context.company_id().is_some() {
        if !user_context.is_manager_or_admin() {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                "Insufficient permissions to update location",
            )));
        }
        location_repo
            .get_all_teams_for_company(user_context.company_id().unwrap())
            .await
    } else {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error(
            "Location ID or company ID must be provided",
        )));
    };

    match teams {
        Ok(teams) => Ok(HttpResponse::Ok().json(ApiResponse::success(teams))),
        Err(err) => {
            log::error!("Error fetching teams: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch teams")))
        }
    }
}

pub async fn get_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    match location_repo.find_by_team_id(team_id).await {
        Ok(Some(location)) => {
            if user_context.company_id() != Some(location.company_id) {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to access team",
                )));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Team not found")));
        }
        Err(err) => {
            log::error!("Error fetching team {}: {}", team_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch team")));
        }
    };

    match location_repo.get_team_by_id(team_id).await {
        Ok(Some(team)) => Ok(HttpResponse::Ok().json(ApiResponse::success(team))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error("Team not found"))),
        Err(err) => {
            log::error!("Error fetching team {}: {}", team_id, err);
            Ok(
                HttpResponse::InternalServerError()
                    .json(ApiResponse::error("Failed to fetch team")),
            )
        }
    }
}

pub async fn update_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
    input: web::Json<TeamInput>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    // Get the location to verify permissions
    match location_repo.find_by_team_id(team_id).await {
        Ok(Some(location)) => {
            // Check if user has access to this location
            if user_context.company_id() != Some(location.company_id)
                || !user_context.is_manager_or_admin()
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to access location",
                )));
            }
            location
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
        }
        Err(err) => {
            log::error!("Error fetching location {}: {}", input.location_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch location")));
        }
    };

    match location_repo.update_team(team_id, input.into_inner()).await {
        Ok(Some(team)) => Ok(HttpResponse::Ok().json(ApiResponse::success(team))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error("Team not found"))),
        Err(err) => {
            log::error!("Error updating team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to update team")))
        }
    }
}

pub async fn delete_team(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    // Get the location to verify permissions
    match location_repo.find_by_team_id(team_id).await {
        Ok(Some(location)) => {
            // Check if user has access to this location
            if user_context.company_id() != Some(location.company_id)
                || !user_context.is_manager_or_admin()
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to access location",
                )));
            }
            location
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
        }
        Err(err) => {
            log::error!("Error fetching location by team {}: {}", team_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch location")));
        }
    };

    match location_repo.delete_team(team_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success("Team deleted successfully"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error("Team not found"))),
        Err(err) => {
            log::error!("Error deleting team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to delete team")))
        }
    }
}

// Team member handlers
pub async fn add_team_member(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse> {
    let (team_id, user_id) = path.into_inner();

    // Get the location to verify permissions
    match location_repo.find_by_team_id(team_id).await {
        Ok(Some(location)) => {
            // Check if user has access to this location
            if user_context.company_id() != Some(location.company_id)
                || !user_context.is_manager_or_admin()
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to access location",
                )));
            }
            location
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
        }
        Err(err) => {
            log::error!("Error fetching location by team {}: {}", team_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch location")));
        }
    };

    match location_repo.add_team_member(team_id, user_id).await {
        Ok(team_member) => Ok(HttpResponse::Created().json(ApiResponse::success(team_member))),
        Err(err) => {
            log::error!("Error adding team member: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to add team member")))
        }
    }
}

pub async fn get_team_members(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    // Get the location to verify permissions
    match location_repo.find_by_team_id(team_id).await {
        Ok(Some(location)) => {
            // Check if user has access to this location
            if user_context.company_id() != Some(location.company_id)
                || !user_context.is_manager_or_admin()
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to access location",
                )));
            }
            location
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
        }
        Err(err) => {
            log::error!("Error fetching location by team {}: {}", team_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch location")));
        }
    };

    // Fetch team members
    match location_repo.get_team_members(team_id).await {
        Ok(members) => Ok(HttpResponse::Ok().json(ApiResponse::success(members))),
        Err(err) => {
            log::error!("Error fetching team members for team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch team members")))
        }
    }
}

pub async fn remove_team_member(
    AsyncUserContext(user_context): AsyncUserContext,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse> {
    let (team_id, user_id) = path.into_inner();

    // Get the location to verify permissions
    match location_repo.find_by_team_id(team_id).await {
        Ok(Some(location)) => {
            // Check if user has access to this location
            if user_context.company_id() != Some(location.company_id)
                || !user_context.is_manager_or_admin()
            {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Insufficient permissions to access location",
                )));
            }
            location
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error("Location not found")));
        }
        Err(err) => {
            log::error!("Error fetching location by team {}: {}", team_id, err);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch location")));
        }
    };

    match location_repo.remove_team_member(team_id, user_id).await {
        Ok(true) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success("Team member removed successfully")))
        }
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error("Team member not found"))),
        Err(err) => {
            log::error!("Error removing team member: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to remove team member")))
        }
    }
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
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to access users",
        )));
    }

    // Get the company ID from user context
    let company_id = match user_context.company_id() {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::BadRequest()
                .json(ApiResponse::error("User does not belong to any company")));
        }
    };

    match company_repo.get_company_employees(company_id).await {
        Ok(employees) => {
            let user_responses: Vec<UserResponse> = employees
                .into_iter()
                .map(|employee| UserResponse {
                    id: employee.id,
                    email: employee.email,
                    name: employee.name,
                    role: employee.role.to_string(),
                    company_id,
                    hire_date: employee.hire_date.map(|d| d.format("%Y-%m-%d").to_string()),
                    is_primary: employee.is_primary,
                    created_at: employee
                        .created_at
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                        .unwrap_or_else(|| {
                            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
                        }),
                    updated_at: employee
                        .updated_at
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                        .unwrap_or_else(|| {
                            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
                        }),
                })
                .collect();

            Ok(HttpResponse::Ok().json(ApiResponse::success(user_responses)))
        }
        Err(err) => {
            log::error!("Error fetching company employees: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch users")))
        }
    }
}

pub async fn update_user(
    claims: Claims,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<String>,
    input: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    let user_id_to_update = match path.into_inner().parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::error("Invalid user ID")));
        }
    };
    let user_id = claims.sub;
    let update_request = input.into_inner();

    // Get company_id for permission check
    let company_id = if let Some(company_id) = update_request.company_id {
        company_id
    } else {
        // Get user's primary company
        match company_repo.get_primary_company_for_user(user_id).await {
            Ok(Some(company_info)) => company_info.id,
            Ok(None) => {
                return Ok(
                    HttpResponse::BadRequest().json(ApiResponse::error("No primary company found"))
                );
            }
            Err(err) => {
                log::error!(
                    "Error fetching primary company for user {}: {}",
                    claims.sub,
                    err
                );
                return Ok(HttpResponse::InternalServerError()
                    .json(ApiResponse::error("Failed to fetch primary company")));
            }
        }
    };

    // Check if user is admin or manager for this specific company
    match company_repo
        .check_user_company_manager_or_admin(user_id, company_id)
        .await
    {
        Ok(true) => {
            // User has permissions, proceed
        }
        Ok(false) => {
            return Ok(
                HttpResponse::Forbidden().json(ApiResponse::error("Insufficient permissions"))
            );
        }
        Err(err) => {
            log::error!(
                "Error checking user permissions for company {}: {}",
                company_id,
                err
            );
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to verify permissions")));
        }
    }

    // Update basic user information
    match user_repo
        .update_user(
            user_id_to_update,
            &update_request.name,
            &update_request.email,
        )
        .await
    {
        Ok(()) => {
            // If role is provided, update company-specific role
            if let (Some(role), Some(company_id)) =
                (&update_request.role, update_request.company_id)
            {
                // Only admins can change roles
                match company_repo
                    .check_user_company_admin(user_id, company_id)
                    .await
                {
                    Ok(true) => {
                        // User is admin, can proceed
                    }
                    Ok(false) => {
                        return Ok(HttpResponse::Forbidden()
                            .json(ApiResponse::error("Only admins can change user roles")));
                    }
                    Err(err) => {
                        log::error!(
                            "Error checking admin permissions for user {}: {}",
                            claims.sub,
                            err
                        );
                        return Ok(HttpResponse::InternalServerError()
                            .json(ApiResponse::error("Failed to verify admin permissions")));
                    }
                }

                // Parse the role
                let company_role = match role.parse::<CompanyRole>() {
                    Ok(role) => role,
                    Err(_) => {
                        return Ok(HttpResponse::BadRequest()
                            .json(ApiResponse::error("Invalid role specified")));
                    }
                };

                // Check if we're trying to modify an admin user (only admins can modify other admins)
                match company_repo.get_company_employees(company_id).await {
                    Ok(employees) => {
                        if let Some(existing_employee) = employees.iter().find(|e| e.id == user_id)
                        {
                            if existing_employee.role == CompanyRole::Admin {
                                // Trying to modify admin - need to be admin
                                match company_repo
                                    .check_user_company_admin(user_id, company_id)
                                    .await
                                {
                                    Ok(true) => {
                                        // User is admin, can proceed
                                    }
                                    Ok(false) => {
                                        return Ok(HttpResponse::Forbidden().json(
                                            ApiResponse::error("Cannot modify admin users"),
                                        ));
                                    }
                                    Err(err) => {
                                        log::error!("Error verifying admin status: {}", err);
                                        return Ok(HttpResponse::InternalServerError().json(
                                            ApiResponse::error("Failed to verify user role"),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Error checking existing user role: {}", err);
                        return Ok(HttpResponse::InternalServerError()
                            .json(ApiResponse::error("Failed to verify user role")));
                    }
                }

                // Update the role in company_employees table
                match company_repo
                    .update_employee_role(company_id, user_id, &company_role)
                    .await
                {
                    Ok(true) => Ok(HttpResponse::Ok().json(
                        ApiResponse::<()>::success_with_message(None, "User updated successfully"),
                    )),
                    Ok(false) => Ok(HttpResponse::NotFound()
                        .json(ApiResponse::error("User not found in company"))),
                    Err(err) => {
                        log::error!("Error updating user role {}: {}", user_id, err);
                        Ok(HttpResponse::InternalServerError()
                            .json(ApiResponse::error("Failed to update user role")))
                    }
                }
            } else {
                Ok(
                    HttpResponse::Ok().json(ApiResponse::<()>::success_with_message(
                        None,
                        "User updated successfully",
                    )),
                )
            }
        }
        Err(err) => {
            log::error!("Error updating user {}: {}", user_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to update user")))
        }
    }
}

pub async fn delete_user(
    AsyncUserContext(user_context): AsyncUserContext,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id_to_delete = path.into_inner();

    // Only admins can delete users - check company-specific admin role
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to delete user",
        )));
    }

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
        Ok(()) => Ok(
            HttpResponse::Ok().json(ApiResponse::<()>::success_with_message(
                None,
                "User deleted successfully",
            )),
        ),
        Err(err) => {
            log::error!("Error deleting user {}: {}", user_id_to_delete, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to delete user")))
        }
    }
}
