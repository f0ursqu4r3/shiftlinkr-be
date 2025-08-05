use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::models::{
    Action, ProficiencyLevel, ShiftRequiredSkillInput, SkillInput, UserSkillInput,
};
use crate::database::repositories::{
    company as company_repo, shift as shift_repo, skill as skill_repo,
};
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::{activity_logger, user_context::extract_context};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserSkillRequest {
    pub proficiency_level: ProficiencyLevel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSearchQuery {
    pub min_level: Option<ProficiencyLevel>,
}

// Skills management
pub async fn create_skill(input: web::Json<SkillInput>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;

    let skill = skill_repo::create_skill(company_id, input.into_inner())
        .await
        .map_err(|e| {
            log::error!("Failed to create skill: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("skill_id", skill.id.to_string()),
        ("skill_name", skill.name.clone()),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        user_context.company_id().unwrap_or_default(),
        Some(user_context.user.id),
        skill.id,
        Action::CREATED,
        "Skill created".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log skill creation activity: {}", e);
    }

    Ok(ApiResponse::success(skill))
}

pub async fn get_all_skills(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let company_id = user_context.strict_company_id()?;

    let skills = skill_repo::get_all_skills(company_id).await.map_err(|e| {
        log::error!("Failed to fetch skills: {}", e);
        AppError::DatabaseError(e)
    })?;

    Ok(ApiResponse::success(skills))
}

pub async fn get_skill(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let skill_id = path.into_inner();
    let company_id = user_context.strict_company_id()?;

    let skill = skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch skill: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Skill not found: {}", skill_id);
            AppError::NotFound("Skill not found".to_string())
        })?;

    Ok(ApiResponse::success(skill))
}

pub async fn update_skill(
    path: web::Path<Uuid>,
    input: web::Json<SkillInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;
    let skill_id = path.into_inner();

    let skill = skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch skill for update: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Skill not found for update: {}", skill_id);
            AppError::NotFound("Skill not found".to_string())
        })?;

    let updated_skill = skill_repo::update_skill(skill_id, input.into_inner())
        .await
        .map_err(|e| {
            log::error!("Failed to update skill: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Skill not found for update: {}", skill_id);
            AppError::NotFound("Skill not found".to_string())
        })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("skill_id", updated_skill.id.to_string()),
        ("skill_name", updated_skill.name.clone()),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        company_id,
        Some(user_context.user.id),
        updated_skill.id,
        Action::UPDATED,
        "Skill updated".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log skill update activity: {}", e);
    }

    Ok(ApiResponse::success(skill))
}

pub async fn delete_skill(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;
    let skill_id = path.into_inner();

    let skill = skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch skill for deletion: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Skill not found for deletion: {}", skill_id);
            AppError::NotFound("Skill not found".to_string())
        })?;

    skill_repo::delete_skill(skill_id).await.map_err(|e| {
        log::error!("Failed to delete skill: {}", e);
        AppError::DatabaseError(e)
    })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("skill_id", skill.id.to_string()),
        ("skill_name", skill.name.clone()),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        company_id,
        Some(user_context.user.id),
        skill.id,
        Action::UPDATED,
        "Skill deleted".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log skill deletion activity: {}", e);
    }

    Ok(ApiResponse::success_message("Skill deleted successfully"))
}

// User Skills management
pub async fn add_user_skill(
    input: web::Json<UserSkillInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;

    let user_id = user_context.user.id;
    let company_id = user_context.strict_company_id()?;
    let (target_user_id, skill_id) = (input.user_id, input.skill_id);

    company_repo::check_user_company_access(target_user_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to check user company access: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("User does not belong to the company: {}", target_user_id);
            AppError::Forbidden("User does not belong to this company".to_string())
        })?;

    skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch skill for user skill creation: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!(
                "Skill not found for user skill creation: {}",
                input.skill_id
            );
            AppError::NotFound("Skill not found".to_string())
        })?;

    let user_skill =
        skill_repo::add_skill_to_user(skill_id, target_user_id, input.proficiency_level.clone())
            .await
            .map_err(|e| {
                log::error!("Failed to add user skill: {}", e);
                AppError::DatabaseError(e)
            })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("user_skill_id", user_skill.id.to_string()),
        ("user_id", user_skill.user_id.to_string()),
        ("skill_id", user_skill.skill_id.to_string()),
        (
            "proficiency_level",
            user_skill.proficiency_level.to_string(),
        ),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        company_id,
        Some(user_id),
        user_skill.id,
        Action::CREATED,
        "User skill added".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log user skill addition activity: {}", e);
    }

    Ok(ApiResponse::success(user_skill))
}

pub async fn get_user_skills(path: web::Path<Uuid>, req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = path.into_inner();

    user_context.requires_same_user(user_id.clone())?;

    let company_id = user_context.strict_company_id()?;

    let user_skills = skill_repo::get_user_skills(user_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch user skills: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(user_skills))
}

pub async fn update_user_skill(
    path: web::Path<(Uuid, Uuid)>,
    input: web::Json<UpdateUserSkillRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let user_id = user_context.user_id();
    let company_id = user_context.strict_company_id()?;
    let (target_user_id, skill_id) = path.into_inner();
    let proficiency_level = input.proficiency_level.clone();

    skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch skill for user skill update: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Skill not found for user skill update: {}", skill_id);
            AppError::NotFound("Skill not found".to_string())
        })?;

    company_repo::check_user_company_access(target_user_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to check user company access: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("User does not belong to the company: {}", target_user_id);
            AppError::Forbidden("User does not belong to this company".to_string())
        })?;

    let updated_user_skill =
        skill_repo::update_user_skill(target_user_id, skill_id, proficiency_level)
            .await
            .map_err(|e| {
                log::error!("Failed to update user skill: {}", e);
                AppError::DatabaseError(e)
            })?
            .ok_or_else(|| {
                log::warn!(
                    "User skill not found for update: {} - {}",
                    target_user_id,
                    skill_id
                );
                AppError::NotFound("User skill not found".to_string())
            })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("user_skill_id", updated_user_skill.id.to_string()),
        ("user_id", updated_user_skill.user_id.to_string()),
        ("skill_id", updated_user_skill.skill_id.to_string()),
        (
            "proficiency_level",
            updated_user_skill.proficiency_level.to_string(),
        ),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        company_id,
        Some(user_id),
        updated_user_skill.id,
        Action::UPDATED,
        "User skill updated".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log user skill update activity: {}", e);
    }

    Ok(ApiResponse::success(updated_user_skill))
}

pub async fn remove_user_skill(
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let user_id = user_context.user_id();
    let company_id = user_context.strict_company_id()?;
    let (target_user_id, skill_id) = path.into_inner();

    company_repo::check_user_company_access(target_user_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to check user company access: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("User does not belong to the company: {}", target_user_id);
            AppError::Forbidden("User does not belong to this company".to_string())
        })?;

    skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch skill for user skill removal: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Skill not found for user skill removal: {}", skill_id);
            AppError::NotFound("Skill not found".to_string())
        })?;

    skill_repo::remove_skill_from_user(skill_id, target_user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to remove user skill: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("user_id", target_user_id.to_string()),
        ("skill_id", skill_id.to_string()),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        company_id,
        Some(user_id),
        skill_id,
        Action::DELETED,
        "User skill removed".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log user skill removal activity: {}", e);
    }

    Ok(ApiResponse::success_message(
        "User skill removed successfully",
    ))
}

// Shift Required Skills management
pub async fn add_shift_required_skill(
    input: web::Json<ShiftRequiredSkillInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;

    skill_repo::find_by_id(input.skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to fetch skill for shift required skill addition: {}",
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!(
                "Skill not found for shift required skill addition: {}",
                input.skill_id
            );
            AppError::NotFound("Skill not found".to_string())
        })?;

    let shift_skill = skill_repo::add_shift_required_skill(
        input.shift_id,
        input.skill_id,
        input.required_level.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Failed to add shift required skill: {}", e);
        AppError::DatabaseError(e)
    })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("shift_required_skill_id", shift_skill.id.to_string()),
        ("shift_id", shift_skill.shift_id.to_string()),
        ("skill_id", shift_skill.skill_id.to_string()),
        ("required_level", shift_skill.required_level.to_string()),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        company_id,
        Some(user_context.user.id),
        shift_skill.id,
        Action::CREATED,
        "Shift required skill added".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!(
            "Failed to log shift required skill addition activity: {}",
            e
        );
    }

    Ok(ApiResponse::created(shift_skill))
}

pub async fn get_shift_required_skills(
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let shift_id = path.into_inner();
    let company_id = user_context.strict_company_id()?;

    shift_repo::find_by_id(shift_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shift for required skills: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Shift not found for required skills: {}", shift_id);
            AppError::NotFound("Shift not found".to_string())
        })?;

    let shift_skills = skill_repo::get_shift_required_skills(shift_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch shift required skills: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(shift_skills))
}

pub async fn remove_shift_required_skill(
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;

    let (shift_id, skill_id) = path.into_inner();

    shift_repo::find_by_id(shift_id, company_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to fetch skill for shift required skill removal: {}",
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!(
                "Skill not found for shift required skill removal: {}",
                skill_id
            );
            AppError::NotFound("Skill not found".to_string())
        })?;

    skill_repo::remove_shift_required_skill(shift_id, skill_id)
        .await
        .map_err(|e| {
            log::error!("Failed to remove shift required skill: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Log the activity
    let metadata = activity_logger::metadata(vec![
        ("shift_id", shift_id.to_string()),
        ("skill_id", skill_id.to_string()),
    ]);

    if let Err(e) = activity_logger::log_skill_activity(
        company_id,
        Some(user_context.user.id),
        skill_id,
        Action::DELETED,
        "Shift required skill removed".to_string(),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log shift required skill removal activity: {}", e);
    }

    Ok(ApiResponse::success_message(
        "Shift required skill removed successfully",
    ))
}

// Skill search and matching
pub async fn get_users_with_skill(
    path: web::Path<Uuid>,
    query: web::Query<SkillSearchQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;
    let skill_id = path.into_inner();
    let min_level = query.min_level.clone();

    skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch skill for user search: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Skill not found for user search: {}", skill_id);
            AppError::NotFound("Skill not found".to_string())
        })?;

    let users = skill_repo::get_users_with_skill(skill_id, min_level)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch users with skill: {}", e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(users))
}
