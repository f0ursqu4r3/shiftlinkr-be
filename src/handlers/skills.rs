use actix_web::{HttpResponse, Result, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        models::{Action, ProficiencyLevel, ShiftRequiredSkillInput, SkillInput, UserSkillInput},
        repositories::{company as company_repo, shift as shifts_repo, skill as skill_repo},
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::request_info::RequestInfo,
    services::{activity_logger, user_context::UserContext},
};

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
pub async fn create_skill(
    ctx: UserContext,
    input: web::Json<SkillInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;
    let user_id = ctx.user_id();

    let skill = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let skill = skill_repo::create_skill(tx, company_id, input.into_inner()).await?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("skill_id", skill.id.to_string()),
                ("skill_name", skill.name.clone()),
            ]);

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                skill.id,
                &Action::CREATED,
                "Skill created".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(skill)
        })
    })
    .await?;

    Ok(ApiResponse::success(skill))
}

pub async fn get_all_skills(ctx: UserContext) -> Result<HttpResponse> {
    let company_id = ctx.strict_company_id()?;

    let skills = skill_repo::get_all_skills(company_id)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(skills))
}

pub async fn get_skill(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let skill_id = path.into_inner();
    let company_id = ctx.strict_company_id()?;

    let skill = skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

    Ok(ApiResponse::success(skill))
}

pub async fn update_skill(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<SkillInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;
    let user_id = ctx.user_id();
    let skill_id = path.into_inner();

    let updated_skill = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let _skill = skill_repo::find_by_id(skill_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            let updated_skill = skill_repo::update_skill(tx, skill_id, input.into_inner())
                .await?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("skill_id", updated_skill.id.to_string()),
                ("skill_name", updated_skill.name.clone()),
            ]);

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                updated_skill.id,
                &Action::UPDATED,
                "Skill updated".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(updated_skill)
        })
    })
    .await?;

    Ok(ApiResponse::success(updated_skill))
}

pub async fn delete_skill(
    path: web::Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;
    let user_id = ctx.user_id();
    let skill_id = path.into_inner();

    let _skill_name = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let skill = skill_repo::find_by_id(skill_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            skill_repo::delete_skill(tx, skill_id).await?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("skill_id", skill.id.to_string()),
                ("skill_name", skill.name.clone()),
            ]);

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                skill.id,
                &Action::DELETED,
                "Skill deleted".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(skill.name)
        })
    })
    .await?;

    Ok(ApiResponse::success_message("Skill deleted successfully"))
}

// User Skills management
pub async fn add_user_skill(
    ctx: UserContext,
    input: web::Json<UserSkillInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let (target_user_id, skill_id) = (input.user_id, input.skill_id);
    let proficiency_level = input.proficiency_level.clone();

    let user_skill = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            company_repo::check_user_company_access(target_user_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| {
                    AppError::Forbidden("User does not belong to this company".to_string())
                })?;

            skill_repo::find_by_id(skill_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            let user_skill =
                skill_repo::add_skill_to_user(tx, skill_id, target_user_id, proficiency_level)
                    .await?;

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

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                user_skill.id,
                &Action::CREATED,
                "User skill added".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(user_skill)
        })
    })
    .await?;

    Ok(ApiResponse::success(user_skill))
}

pub async fn get_user_skills(path: web::Path<Uuid>, ctx: UserContext) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    ctx.requires_same_user(user_id)?;

    let company_id = ctx.strict_company_id()?;

    let user_skills = skill_repo::get_user_skills(user_id, company_id)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(user_skills))
}

pub async fn update_user_skill(
    path: web::Path<(Uuid, Uuid)>,
    ctx: UserContext,
    input: web::Json<UpdateUserSkillRequest>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let (target_user_id, skill_id) = path.into_inner();
    let proficiency_level = input.proficiency_level.clone();

    let updated_user_skill = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            skill_repo::find_by_id(skill_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            company_repo::check_user_company_access(target_user_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| {
                    AppError::Forbidden("User does not belong to this company".to_string())
                })?;

            let updated_user_skill =
                skill_repo::update_user_skill(tx, target_user_id, skill_id, proficiency_level)
                    .await?
                    .ok_or_else(|| AppError::NotFound("User skill not found".to_string()))?;

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

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                updated_user_skill.id,
                &Action::UPDATED,
                "User skill updated".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(updated_user_skill)
        })
    })
    .await?;

    Ok(ApiResponse::success(updated_user_skill))
}

pub async fn remove_user_skill(
    path: web::Path<(Uuid, Uuid)>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let (target_user_id, skill_id) = path.into_inner();

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            skill_repo::find_by_id(skill_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            company_repo::check_user_company_access(target_user_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| {
                    AppError::Forbidden("User does not belong to this company".to_string())
                })?;

            // Get the user skill before deleting it for logging
            let user_skills = skill_repo::get_user_skills(target_user_id, company_id)
                .await
                .map_err(AppError::from)?;

            let user_skill = user_skills
                .into_iter()
                .find(|us| us.skill_id == skill_id)
                .ok_or_else(|| AppError::NotFound("User skill not found".to_string()))?;

            skill_repo::remove_skill_from_user(tx, skill_id, target_user_id)
                .await?
                .ok_or_else(|| AppError::NotFound("User skill not found".to_string()))?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("user_id", target_user_id.to_string()),
                ("skill_id", skill_id.to_string()),
            ]);

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                user_skill.id,
                &Action::DELETED,
                "User skill removed".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    Ok(ApiResponse::success("User skill removed successfully"))
}

// Shift Required Skills management
pub async fn add_shift_required_skill(
    ctx: UserContext,
    input: web::Json<ShiftRequiredSkillInput>,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let shift_id = input.shift_id;
    let skill_id = input.skill_id;
    let required_level = input.required_level.clone();

    let shift_skill = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            skill_repo::find_by_id(skill_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            shifts_repo::find_by_id(shift_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            let shift_skill =
                skill_repo::add_shift_required_skill(tx, shift_id, skill_id, required_level)
                    .await?;

            // Log the activity
            let metadata = activity_logger::metadata(vec![
                ("shift_id", shift_id.to_string()),
                ("skill_id", skill_id.to_string()),
            ]);

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                shift_skill.id,
                &Action::CREATED,
                "Required skill added to shift".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(shift_skill)
        })
    })
    .await?;

    Ok(ApiResponse::success(shift_skill))
}

pub async fn get_shift_required_skills(
    path: web::Path<Uuid>,
    ctx: UserContext,
) -> Result<HttpResponse> {
    let company_id = ctx.strict_company_id()?;
    let shift_id = path.into_inner();

    shifts_repo::find_by_id(shift_id, company_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

    let required_skills = skill_repo::get_shift_required_skills(shift_id)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(required_skills))
}

pub async fn remove_shift_required_skill(
    path: web::Path<(Uuid, Uuid)>,
    ctx: UserContext,
    req_info: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let user_id = ctx.user_id();
    let company_id = ctx.strict_company_id()?;
    let (shift_id, skill_id) = path.into_inner();

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            skill_repo::find_by_id(skill_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

            shifts_repo::find_by_id(shift_id, company_id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::NotFound("Shift not found".to_string()))?;

            skill_repo::remove_shift_required_skill(tx, shift_id, skill_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Shift required skill not found".to_string()))?;

            // Log the activity - using skill_id as entity_id since the shift skill was removed
            let metadata = activity_logger::metadata(vec![
                ("shift_id", shift_id.to_string()),
                ("skill_id", skill_id.to_string()),
            ]);

            activity_logger::log_skill_activity(
                tx,
                company_id,
                Some(user_id),
                skill_id, // Use skill_id as entity_id for logging
                &Action::DELETED,
                "Required skill removed from shift".to_string(),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    Ok(ApiResponse::success("Required skill removed from shift"))
}

// Skill search and matching
pub async fn get_users_with_skill(
    path: web::Path<Uuid>,
    query: web::Query<SkillSearchQuery>,
    ctx: UserContext,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;
    let company_id = ctx.strict_company_id()?;
    let skill_id = path.into_inner();
    let min_level = query.min_level.clone();

    skill_repo::find_by_id(skill_id, company_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Skill not found".to_string()))?;

    let users = skill_repo::get_users_with_skill(skill_id, min_level)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(users))
}
