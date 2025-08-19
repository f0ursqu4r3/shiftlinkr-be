use actix_web::{HttpResponse, Result, web};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        models::{
            Action, CompanyRole, CreateUpdateLocationInput, CreateUpdateTeamInput, LocationInput,
        },
        repositories::{
            company as company_repo, location as location_repo, team as team_repo,
            user as user_repo,
        },
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::request_info::RequestInfo,
    services::{activity_logger, user_context::UserContext},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserInput {
    pub name: String,
    pub email: String,
    pub role: Option<CompanyRole>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: CompanyRole,
    pub company_id: Uuid,
    pub hire_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Location handlers
pub async fn create_location(
    ctx: UserContext,
    input: web::Json<CreateUpdateLocationInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let location_input = input.into_inner();

    // Extract values that need to be moved
    let name = location_input.name;
    let address = location_input.address;
    let phone = location_input.phone;
    let email = location_input.email;

    let location = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let location = location_repo::create_location(
                tx,
                LocationInput {
                    name: name.clone(),
                    address,
                    phone,
                    email,
                    company_id,
                },
            )
            .await?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("location_name", location.name.clone()),
                ("location_id", location.id.to_string()),
            ]);
            activity_logger::log_location_activity(
                tx,
                company_id,
                Some(user_id),
                location.id,
                Action::CREATED,
                format!("Location '{}' created", location.name),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(location)
        })
    })
    .await?;

    Ok(ApiResponse::created(location))
}

pub async fn get_locations(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let companies = company_repo::get_companies_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching user companies: {}", e);
            AppError::DatabaseError(e)
        })?;

    if companies.is_empty() {
        return Ok(ApiResponse::success(Vec::<LocationInput>::new()));
    }

    let company_ids: Vec<Uuid> = companies.iter().map(|c| c.id).collect();

    let all_locations = location_repo::get_locations_by_company_ids(company_ids)
        .await
        .map_err(|e| {
            log::error!("Error fetching locations: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(all_locations))
}

pub async fn get_location(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let location_id = path.into_inner();
    let user_id = ctx.user_id();

    let location = location_repo::find_by_id(location_id)
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
    company_repo::check_user_company_access(user_id, location.company_id)
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
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<CreateUpdateLocationInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    let company_id = ctx.strict_company_id()?;

    ctx.requires_manager()?;

    let location_id = path.into_inner();

    let location = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let location = location_repo::update_location(
                tx,
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

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("location_name", location.name.clone()),
                ("location_id", location.id.to_string()),
            ]);
            activity_logger::log_location_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                location.id,
                Action::UPDATED,
                format!("Location '{}' updated", location.name),
                Some(metadata),
                &req_info,
            )
            .await?;
            Ok(location)
        })
    })
    .await?;

    Ok(ApiResponse::success(location))
}

pub async fn delete_location(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    let location_id = path.into_inner();

    // Get the location to check permissions
    let location = location_repo::find_by_id(location_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location {}: {}", location_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location {} not found", location_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    ctx.requires_same_company(location.company_id)?;

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Check if the location has any teams
            let teams = team_repo::get_teams_by_location(location_id)
                .await
                .map_err(|e| {
                    log::error!("Error fetching teams for location {}: {}", location_id, e);
                    AppError::DatabaseError(e)
                })?;

            if !teams.is_empty() {
                return Err(AppError::BadRequest(
                    "Cannot delete location with existing teams".to_string(),
                ));
            }

            // Proceed with deletion
            location_repo::delete_location(tx, location_id).await?;

            // Log the activity
            let metadata =
                activity_logger::metadata(vec![("location_id", location_id.to_string())]);

            activity_logger::log_location_activity(
                tx,
                location.company_id,
                Some(ctx.user_id()),
                location.id,
                Action::DELETED,
                format!("Location '{}' deleted", location.name),
                Some(metadata),
                &req_info,
            )
            .await?;
            Ok(())
        })
    })
    .await?;

    Ok(ApiResponse::success_message(
        "Location deleted successfully",
    ))
}

// Team handlers
pub async fn create_team(
    ctx: UserContext,
    input: web::Json<CreateUpdateTeamInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    // Get the location to determine which company it belongs to
    let location_id = input.location_id;
    let team_input = input.into_inner();
    let team_name = team_input.name.clone();
    let user_id = ctx.user_id();

    let location = location_repo::find_by_id(location_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location {}: {}", location_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location {} not found", location_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    ctx.requires_same_company(location.company_id)?;

    let team = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let team = team_repo::create_team(tx, team_input).await.map_err(|e| {
                log::error!("Error creating team: {}", e);
                AppError::DatabaseError(e)
            })?;

            let metadata = activity_logger::metadata(vec![
                ("team_name", team_name),
                ("team_id", team.id.to_string()),
                ("location_id", location_id.to_string()),
            ]);

            activity_logger::log_team_activity(
                tx,
                location.company_id,
                Some(user_id),
                team.id,
                Action::CREATED,
                format!("Team '{}' created in location {}", team.name, location_id),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(team)
        })
    })
    .await?;

    Ok(ApiResponse::created(team))
}

pub async fn get_teams(query: web::Query<TeamQuery>, ctx: UserContext) -> Result<HttpResponse> {
    let teams = if let Some(location_id) = query.location_id {
        let location = location_repo::find_by_id(location_id)
            .await
            .map_err(|e| {
                log::error!("Error fetching location {}: {}", location_id, e);
                AppError::DatabaseError(e)
            })?
            .ok_or_else(|| {
                log::warn!("Location {} not found", location_id);
                AppError::NotFound("Location not found".to_string())
            })?;

        ctx.requires_same_company(location.company_id)?;

        team_repo::get_teams_by_location(location_id)
            .await
            .map_err(|e| {
                log::error!("Error fetching teams for location {}: {}", location_id, e);
                AppError::DatabaseError(e)
            })?
    } else {
        ctx.requires_manager()?;
        let company_id = ctx.strict_company_id()?;

        team_repo::get_all_teams_for_company(company_id)
            .await
            .map_err(|e| {
                log::error!("Error fetching teams for company: {}", e);
                AppError::DatabaseError(e)
            })?
    };

    Ok(ApiResponse::success(teams))
}

pub async fn get_team(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    ctx.requires_manager()?;

    let location = get_location_for_team(team_id).await?;

    ctx.requires_same_company(location.company_id)?;

    let team = team_repo::get_team_by_id(team_id)
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
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<CreateUpdateTeamInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let team_id = path.into_inner();

    ctx.requires_manager()?;

    let location = location_repo::find_by_team_id(team_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching location by team {}: {}", team_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Location for team {} not found", team_id);
            AppError::NotFound("Location not found".to_string())
        })?;

    ctx.requires_same_company(location.company_id)?;

    let team = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let team = team_repo::update_team(tx, team_id, input.into_inner())
                .await?
                .ok_or_else(|| {
                    log::warn!("Team {} not found", team_id);
                    AppError::NotFound("Team not found".to_string())
                })?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("team_name", team.name.clone()),
                ("team_id", team.id.to_string()),
                ("location_id", location.id.to_string()),
            ]);

            activity_logger::log_team_activity(
                tx,
                location.company_id,
                Some(user_id),
                team.id,
                Action::UPDATED,
                format!("Team '{}' updated in location {}", team.name, location.id),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(team)
        })
    })
    .await?;
    Ok(ApiResponse::success(team))
}

pub async fn delete_team(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    ctx.requires_manager()?;

    let location = get_location_for_team(team_id).await?;

    ctx.requires_same_company(location.company_id)?;

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            team_repo::delete_team(tx, team_id).await?.ok_or_else(|| {
                log::warn!("Team {} not found for deletion", team_id);
                AppError::NotFound("Team not found".to_string())
            })?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("team_id", team_id.to_string()),
                ("location_id", location.id.to_string()),
            ]);

            activity_logger::log_team_activity(
                tx,
                location.company_id,
                Some(ctx.user_id()),
                team_id,
                Action::DELETED,
                format!("Team with ID {} deleted", team_id),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;
    Ok(ApiResponse::success_message("Team deleted successfully"))
}

// Team member handlers
pub async fn add_team_member(
    path: web::Path<(Uuid, Uuid)>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    let (team_id, user_id) = path.into_inner();

    ctx.requires_manager()?;

    let location = get_location_for_team(team_id).await?;

    ctx.requires_same_company(location.company_id)?;

    let team_member = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let team_member = team_repo::add_team_member(tx, team_id, user_id)
                .await
                .map_err(|e| {
                    log::error!("Error adding team member to team {}: {}", team_id, e);
                    AppError::DatabaseError(e)
                })?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("team_id", team_id.to_string()),
                ("user_id", user_id.to_string()),
            ]);

            activity_logger::log_team_activity(
                tx,
                location.company_id,
                Some(ctx.user_id()),
                team_id,
                Action::MEMBER_ADDED,
                format!("User {} added to team {}", user_id, team_id),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(team_member)
        })
    })
    .await?;

    Ok(ApiResponse::success(team_member))
}

pub async fn get_team_members(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let team_id = path.into_inner();

    ctx.requires_manager()?;

    let location = get_location_for_team(team_id).await?;

    ctx.requires_same_company(location.company_id)?;

    let members = team_repo::get_team_members(team_id).await.map_err(|e| {
        log::error!("Error fetching team members for team {}: {}", team_id, e);
        AppError::DatabaseError(e)
    })?;

    Ok(ApiResponse::success(members))
}

pub async fn remove_team_member(
    path: web::Path<(Uuid, Uuid)>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    let (team_id, user_id) = path.into_inner();

    ctx.requires_manager()?;

    let location = get_location_for_team(team_id).await?;

    ctx.requires_same_company(location.company_id)?;

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            team_repo::remove_team_member(tx, team_id, user_id)
                .await
                .map_err(|e| {
                    log::error!("Error removing team member from team {}: {}", team_id, e);
                    AppError::DatabaseError(e)
                })?
                .ok_or_else(|| {
                    log::warn!("Team member {} not found in team {}", user_id, team_id);
                    AppError::NotFound("Team member not found".to_string())
                })?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("team_id", team_id.to_string()),
                ("user_id", user_id.to_string()),
            ]);

            activity_logger::log_team_activity(
                tx,
                location.company_id,
                Some(ctx.user_id()),
                team_id,
                Action::MEMBER_REMOVED,
                format!("User {} removed from team {}", user_id, team_id),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

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
pub async fn get_users(ctx: UserContext) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;

    let employees: Vec<UserResponse> = company_repo::get_company_employees(company_id)
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
            hire_date: employee.hire_date,
            created_at: employee.created_at.unwrap_or_else(|| chrono::Utc::now()),
            updated_at: employee.updated_at.unwrap_or_else(|| chrono::Utc::now()),
        })
        .collect();

    Ok(ApiResponse::success(employees))
}

pub async fn update_user(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<UpdateUserInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse, AppError> {
    let user_id_to_update = path.into_inner();
    let user_id = ctx.user_id();
    let update_request = input.into_inner();

    // Check permissions based on whether role is being updated
    if update_request.role.is_some() {
        ctx.requires_manager()?;
    } else {
        ctx.requires_same_user(user_id_to_update)?;
    }

    // Get the company ID from user context
    let company_id = ctx.strict_company_id()?;

    // Check if the user to update belongs to requesting user's company
    let user_company_info =
        company_repo::find_user_company_info_by_id(user_id_to_update, company_id)
            .await?
            .ok_or_else(|| {
                AppError::PermissionDenied("User does not belong to the same company".to_string())
            })?;

    let (updated_user, final_role) = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Update basic user information
            let updated_user = user_repo::update_user(
                tx,
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
                company_repo::update_employee_role(tx, company_id, user_id_to_update, &new_role)
                    .await?;

                new_role
            } else {
                user_company_info.role
            };

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("user_id", user_id_to_update.to_string()),
                ("updated_by", user_id.to_string()),
                ("role", final_role.to_string()),
            ]);

            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(user_id),
                user_id_to_update,
                Action::UPDATED,
                format!("User {} updated", user_id_to_update),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok((updated_user, final_role))
        })
    })
    .await?;

    Ok(ApiResponse::success(UserResponse {
        id: updated_user.id,
        email: updated_user.email,
        name: updated_user.name,
        role: final_role,
        company_id,
        hire_date: user_company_info.hire_date,
        created_at: updated_user.created_at,
        updated_at: updated_user.updated_at,
    }))
}

pub async fn delete_user(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    let user_id_to_delete = path.into_inner();

    // Only admins can delete users - check company-specific admin role
    ctx.requires_manager()?;

    // Get the company ID from user context
    let company_id = ctx.strict_company_id()?;

    // Check if user is admin in the specific company
    let employees = company_repo::get_company_employees(company_id)
        .await
        .map_err(|e| {
            log::error!("Error fetching company employees: {}", e);
            AppError::DatabaseError(e)
        })?;
    // Check if the user to delete is an admin
    let admin_count = employees
        .iter()
        .filter(|e| e.role == CompanyRole::Admin)
        .count();
    if admin_count <= 1 && employees.iter().any(|e| e.id == user_id_to_delete) {
        // If the user to delete is the only admin, prevent deletion
        return Err(
            AppError::PermissionDenied("Cannot delete the only admin user".to_string()).into(),
        );
    }

    // Proceed with deletion
    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            user_repo::delete_user(tx, user_id_to_delete).await?;

            // Log the activity
            let metadata =
                activity_logger::metadata(vec![("user_id", user_id_to_delete.to_string())]);
            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(ctx.user_id()),
                user_id_to_delete,
                Action::DELETED,
                format!("User with ID {} deleted", user_id_to_delete),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    Ok(ApiResponse::success_message("User deleted successfully"))
}

// Utilities
async fn get_location_for_team(
    team_id: Uuid,
) -> Result<crate::database::models::Location, AppError> {
    location_repo::find_by_team_id(team_id)
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
