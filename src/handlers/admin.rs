use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::database::location_repository::LocationRepository;
use crate::database::models::{LocationInput, TeamInput};
use crate::auth::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
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

// Location handlers
pub async fn create_location(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    input: web::Json<LocationInput>,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    match location_repo.create_location(input.into_inner()).await {
        Ok(location) => Ok(HttpResponse::Created().json(ApiResponse::success(location))),
        Err(err) => {
            log::error!("Error creating location: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create location")))
        }
    }
}

pub async fn get_locations(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
) -> Result<HttpResponse> {
    // All authenticated users can view locations
    match location_repo.get_all_locations().await {
        Ok(locations) => Ok(HttpResponse::Ok().json(ApiResponse::success(locations))),
        Err(err) => {
            log::error!("Error fetching locations: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch locations")))
        }
    }
}

pub async fn get_location(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let location_id = path.into_inner();

    match location_repo.get_location_by_id(location_id).await {
        Ok(Some(location)) => Ok(HttpResponse::Ok().json(ApiResponse::success(location))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Location not found"))),
        Err(err) => {
            log::error!("Error fetching location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch location")))
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
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let location_id = path.into_inner();

    match location_repo.update_location(location_id, input.into_inner()).await {
        Ok(Some(location)) => Ok(HttpResponse::Ok().json(ApiResponse::success(location))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Location not found"))),
        Err(err) => {
            log::error!("Error updating location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update location")))
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
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let location_id = path.into_inner();

    match location_repo.delete_location(location_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success("Location deleted successfully"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Location not found"))),
        Err(err) => {
            log::error!("Error deleting location {}: {}", location_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete location")))
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
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    match location_repo.create_team(input.into_inner()).await {
        Ok(team) => Ok(HttpResponse::Created().json(ApiResponse::success(team))),
        Err(err) => {
            log::error!("Error creating team: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create team")))
        }
    }
}

pub async fn get_teams(
    claims: Claims,
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
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch teams")))
        }
    }
}

pub async fn get_team(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    match location_repo.get_team_by_id(team_id).await {
        Ok(Some(team)) => Ok(HttpResponse::Ok().json(ApiResponse::success(team))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team not found"))),
        Err(err) => {
            log::error!("Error fetching team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch team")))
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
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let team_id = path.into_inner();

    match location_repo.update_team(team_id, input.into_inner()).await {
        Ok(Some(team)) => Ok(HttpResponse::Ok().json(ApiResponse::success(team))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team not found"))),
        Err(err) => {
            log::error!("Error updating team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update team")))
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
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let team_id = path.into_inner();

    match location_repo.delete_team(team_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success("Team deleted successfully"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team not found"))),
        Err(err) => {
            log::error!("Error deleting team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete team")))
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
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let (team_id, user_id) = path.into_inner();

    match location_repo.add_team_member(team_id, user_id).await {
        Ok(team_member) => Ok(HttpResponse::Created().json(ApiResponse::success(team_member))),
        Err(err) => {
            log::error!("Error adding team member: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to add team member")))
        }
    }
}

pub async fn get_team_members(
    claims: Claims,
    location_repo: web::Data<LocationRepository>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    match location_repo.get_team_members(team_id).await {
        Ok(members) => Ok(HttpResponse::Ok().json(ApiResponse::success(members))),
        Err(err) => {
            log::error!("Error fetching team members for team {}: {}", team_id, err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch team members")))
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
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions")));
    }

    let (team_id, user_id) = path.into_inner();

    match location_repo.remove_team_member(team_id, user_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success("Team member removed successfully"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Team member not found"))),
        Err(err) => {
            log::error!("Error removing team member: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to remove team member")))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TeamQuery {
    pub location_id: Option<i64>,
}
