use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::auth::Claims;
use crate::database::models::{LocationInput, TeamInput};
use crate::database::repositories::location_repository::LocationRepository;
use crate::database::repositories::user_repository::UserRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub name: String,
    pub email: String,
    // TODO: Role updates will be handled through company_employees table
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub hire_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }
    }
}

impl ApiResponse<()> {
    pub fn success_with_message(message: &str) -> Self {
        Self {
            success: true,
            data: None,
            message: Some(message.to_string()),
        }
    }
}

// Location handlers
pub async fn create_location(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    input: web::Json<LocationInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match location_repo.create_location(input.into_inner()).await {
        Ok(location) => Ok(HttpResponse::Created().json(ApiResponse::success(location))),
        Err(err) => {
            log::error!("Error creating location: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create location")))
        }
    }
}

pub async fn get_locations(
    _claims: Claims,
    location_repo: web::Data<LocationRepository>,
) -> Result<HttpResponse> {
    // All authenticated users can view locations
    match location_repo.get_all_locations().await {
        Ok(locations) => Ok(HttpResponse::Ok().json(ApiResponse::success(locations))),
        Err(err) => {
            log::error!("Error fetching locations: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch locations")))
        }
    }
}

pub async fn get_location(
    _claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let location_id = path.into_inner();

    match location_repo.get_location_by_id(location_id).await {
        Ok(Some(location)) => Ok(HttpResponse::Ok().json(ApiResponse::success(location))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Location not found")))
        }
        Err(err) => {
            log::error!("Error fetching location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch location")))
        }
    }
}

pub async fn update_location(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
    input: web::Json<LocationInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let location_id = path.into_inner();

    match location_repo
        .update_location(location_id, input.into_inner())
        .await
    {
        Ok(Some(location)) => Ok(HttpResponse::Ok().json(ApiResponse::success(location))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Location not found")))
        }
        Err(err) => {
            log::error!("Error updating location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update location")))
        }
    }
}

pub async fn delete_location(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    // Check if user is admin
    if !claims.is_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let location_id = path.into_inner();

    match location_repo.delete_location(location_id).await {
        Ok(true) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success("Location deleted successfully")))
        }
        Ok(false) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Location not found")))
        }
        Err(err) => {
            log::error!("Error deleting location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to delete location")))
        }
    }
}

// Team handlers
pub async fn create_team(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    input: web::Json<TeamInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match location_repo.create_team(input.into_inner()).await {
        Ok(team) => Ok(HttpResponse::Created().json(ApiResponse::success(team))),
        Err(err) => {
            log::error!("Error creating team: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create team")))
        }
    }
}

pub async fn get_teams(
    _claims: Claims,
    location_repo: web::Data<LocationRepository>,
    query: web::Query<TeamQuery>,
) -> Result<HttpResponse> {
    let teams = if let Some(location_id) = query.location_id {
        location_repo.get_teams_by_location(location_id).await
    } else {
        location_repo.get_all_teams().await
    };

    match teams {
        Ok(teams) => Ok(HttpResponse::Ok().json(ApiResponse::success(teams))),
        Err(err) => {
            log::error!("Error fetching teams: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch teams")))
        }
    }
}

pub async fn get_team(
    _claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    match location_repo.get_team_by_id(team_id).await {
        Ok(Some(team)) => Ok(HttpResponse::Ok().json(ApiResponse::success(team))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team not found"))),
        Err(err) => {
            log::error!("Error fetching team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch team")))
        }
    }
}

pub async fn update_team(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
    input: web::Json<TeamInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let team_id = path.into_inner();

    match location_repo.update_team(team_id, input.into_inner()).await {
        Ok(Some(team)) => Ok(HttpResponse::Ok().json(ApiResponse::success(team))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team not found"))),
        Err(err) => {
            log::error!("Error updating team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update team")))
        }
    }
}

pub async fn delete_team(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let team_id = path.into_inner();

    match location_repo.delete_team(team_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success("Team deleted successfully"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team not found"))),
        Err(err) => {
            log::error!("Error deleting team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to delete team")))
        }
    }
}

// Team member handlers
pub async fn add_team_member(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let (team_id, user_id) = path.into_inner();

    match location_repo.add_team_member(team_id, user_id).await {
        Ok(team_member) => Ok(HttpResponse::Created().json(ApiResponse::success(team_member))),
        Err(err) => {
            log::error!("Error adding team member: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to add team member")))
        }
    }
}

pub async fn get_team_members(
    _claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    match location_repo.get_team_members(team_id).await {
        Ok(members) => Ok(HttpResponse::Ok().json(ApiResponse::success(members))),
        Err(err) => {
            log::error!("Error fetching team members for team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch team members")))
        }
    }
}

pub async fn remove_team_member(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let (team_id, user_id) = path.into_inner();

    match location_repo.remove_team_member(team_id, user_id).await {
        Ok(true) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success("Team member removed successfully")))
        }
        Ok(false) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team member not found")))
        }
        Err(err) => {
            log::error!("Error removing team member: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to remove team member")))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TeamQuery {
    pub location_id: Option<i64>,
}

// User management handlers
pub async fn get_users(
    claims: Claims,
    user_repo: web::Data<UserRepository>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    match user_repo.get_all_users().await {
        Ok(users) => {
            let user_responses: Vec<UserResponse> = users
                .into_iter()
                .map(|user| UserResponse {
                    id: user.id,
                    email: user.email,
                    name: user.name,
                    // TODO: Add role from company_employees table based on selected company
                    hire_date: user.hire_date.map(|d| d.format("%Y-%m-%d").to_string()),
                    created_at: user.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    updated_at: user.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                })
                .collect();

            Ok(HttpResponse::Ok().json(ApiResponse::success(user_responses)))
        }
        Err(err) => {
            log::error!("Error fetching users: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch users")))
        }
    }
}

pub async fn update_user(
    claims: Claims,
    user_repo: web::Data<UserRepository>,
    path: web::Path<String>,
    input: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let user_id = path.into_inner();
    let update_request = input.into_inner();

    // Only admins can change user roles or edit other admins
    if !claims.is_admin() {
        // TODO: Check if we're trying to modify an admin user based on company-specific role
        // Since roles are now company-specific, we need to check the role in the context of a specific company
        // For now, allowing managers to edit users
        /*
        if let Ok(Some(existing_user)) = user_repo.find_by_id(&user_id).await {
            if existing_user.role == UserRole::Admin {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Cannot modify admin users")));
            }
        }

        // Managers can't change roles
        if let Ok(Some(existing_user)) = user_repo.find_by_id(&user_id).await {
            if existing_user.role != update_request.role {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Cannot change user roles")));
            }
        }
        */
    }

    match user_repo
        .update_user(&user_id, &update_request.name, &update_request.email)
        .await
    {
        Ok(()) => Ok(HttpResponse::Ok().json(ApiResponse::success_with_message(
            "User updated successfully",
        ))),
        Err(err) => {
            log::error!("Error updating user {}: {}", user_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update user")))
        }
    }
}

pub async fn delete_user(
    claims: Claims,
    user_repo: web::Data<UserRepository>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // Only admins can delete users
    if !claims.is_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"))
        );
    }

    let user_id = path.into_inner();

    // Prevent deleting admin users (including self)
    // TODO: Check if user is admin based on company-specific role
    // Since roles are now company-specific, we need to check the role in the context of a specific company
    // For now, allowing deletion of users
    /*
    if let Ok(Some(user)) = user_repo.find_by_id(&user_id).await {
        if user.role == UserRole::Admin {
            return Ok(HttpResponse::BadRequest()
                .json(ApiResponse::<()>::error("Cannot delete admin users")));
        }
    }
    */

    match user_repo.delete_user(&user_id).await {
        Ok(()) => Ok(HttpResponse::Ok().json(ApiResponse::success_with_message(
            "User deleted successfully",
        ))),
        Err(err) => {
            log::error!("Error deleting user {}: {}", user_id, err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to delete user")))
        }
    }
}
