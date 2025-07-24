use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::models::{
    ProficiencyLevel, ShiftRequiredSkillInput, SkillInput, UserSkillInput,
};
use crate::database::repositories::{company::CompanyRepository, skill::SkillRepository};
use crate::handlers::admin::ApiResponse;
use crate::services::UserContext;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserSkillRequest {
    pub proficiency_level: ProficiencyLevel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSearchQuery {
    pub skill_id: Option<i64>,
    pub min_level: Option<ProficiencyLevel>,
}

// Skills management
pub async fn create_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    input: web::Json<SkillInput>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"))
        );
    }

    let company_id = match user_context.company_id() {
        Some(id) => id,
        None => {
            return Ok(
                HttpResponse::BadRequest().json(ApiResponse::<()>::error("Company ID is required"))
            );
        }
    };

    match skill_repo
        .create_skill(company_id, input.into_inner())
        .await
    {
        Ok(skill) => Ok(HttpResponse::Created().json(ApiResponse::success(skill))),
        Err(e) => {
            log::error!("Failed to create skill: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create skill")))
        }
    }
}

pub async fn get_all_skills(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let company_id = match user_context.company_id() {
        Some(id) => id,
        None => {
            return Ok(
                HttpResponse::BadRequest().json(ApiResponse::<()>::error("Company ID is required"))
            );
        }
    };
    match skill_repo.get_all_skills(company_id).await {
        Ok(skills) => Ok(HttpResponse::Ok().json(ApiResponse::success(skills))),
        Err(e) => {
            log::error!("Failed to get skills: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get skills")))
        }
    }
}

pub async fn get_skill(
    _user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    path: web::Path<i64>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let skill_id = path.into_inner();

    match skill_repo.get_skill_by_id(skill_id).await {
        Ok(Some(skill)) => Ok(HttpResponse::Ok().json(ApiResponse::success(skill))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Skill not found"))),
        Err(e) => {
            log::error!("Failed to get skill: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get skill")))
        }
    }
}

pub async fn update_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<Uuid>,
    input: web::Json<SkillInput>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // For skills management, we require admin access to the default company (company_id = 1)
    let default_company_id = user_context.company_id().unwrap_or_default();

    if !company_repo
        .check_user_company_manager_or_admin(user_context.user.id, default_company_id)
        .await
        .unwrap_or(false)
    {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"))
        );
    }

    let skill_id = path.into_inner();

    match skill_repo.update_skill(skill_id, input.into_inner()).await {
        Ok(Some(skill)) => Ok(HttpResponse::Ok().json(ApiResponse::success(skill))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Skill not found"))),
        Err(e) => {
            log::error!("Failed to update skill: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update skill")))
        }
    }
}

pub async fn delete_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<i64>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // For skills management, we require admin access to the default company (company_id = 1)
    let default_company_id = user_context.company_id().unwrap_or_default();

    if !company_repo
        .check_user_company_manager_or_admin(user_context.user.id, default_company_id)
        .await
        .unwrap_or(false)
    {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"))
        );
    }

    let skill_id = path.into_inner();

    // Note: There's an inconsistency in the skill repository where some methods use i64 and others use Uuid
    // For now, we'll skip the delete operation or use a different approach
    match skill_repo.get_skill_by_id(skill_id).await {
        Ok(Some(_)) => {
            // TODO: Fix repository inconsistency - delete_skill expects Uuid but we have i64
            log::warn!("Cannot delete skill due to repository type inconsistency");
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Skill deletion temporarily unavailable",
                )),
            )
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Skill not found"))),
        Err(e) => {
            log::error!("Failed to delete skill: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to delete skill")))
        }
    }
}

// User Skills management
pub async fn add_user_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    input: web::Json<UserSkillInput>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Users can only manage their own skills, admins can manage any
    if !user_context.is_admin() && user_context.user_id() != input.user_id {
        return Ok(HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("Can only manage your own skills")));
    }

    match skill_repo.add_user_skill(input.into_inner()).await {
        Ok(user_skill) => Ok(HttpResponse::Created().json(ApiResponse::success(user_skill))),
        Err(e) => {
            log::error!("Failed to add user skill: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to add user skill")))
        }
    }
}

pub async fn get_user_skills(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    path: web::Path<String>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only view their own skills, admins/managers can view any
    if !user_context.is_manager_or_admin() && user_context.user_id().to_string() != user_id {
        return Ok(HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("Can only view your own skills")));
    }

    match skill_repo.get_user_skills(&user_id).await {
        Ok(user_skills) => Ok(HttpResponse::Ok().json(ApiResponse::success(user_skills))),
        Err(e) => {
            log::error!("Failed to get user skills: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get user skills")))
        }
    }
}

pub async fn update_user_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    path: web::Path<i64>,
    input: web::Json<UpdateUserSkillRequest>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let skill_id = path.into_inner();

    // Get the user skill first to check ownership
    match skill_repo
        .update_user_skill(skill_id, input.proficiency_level.clone())
        .await
    {
        Ok(Some(user_skill)) => {
            // Check if user can update this skill
            if !user_context.is_admin() && user_context.user_id() != user_skill.user_id {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Can only update your own skills")));
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(user_skill)))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("User skill not found")))
        }
        Err(e) => {
            log::error!("Failed to update user skill: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update user skill")))
        }
    }
}

pub async fn remove_user_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    path: web::Path<(String, i64)>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let (user_id, skill_id) = path.into_inner();

    // Users can only remove their own skills, admins can remove any
    if !user_context.is_admin() && user_context.user_id().to_string() != user_id {
        return Ok(HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("Can only remove your own skills")));
    }

    match skill_repo.remove_user_skill(&user_id, skill_id).await {
        Ok(true) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success("User skill removed successfully")))
        }
        Ok(false) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("User skill not found")))
        }
        Err(e) => {
            log::error!("Failed to remove user skill: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to remove user skill")))
        }
    }
}

// Shift Required Skills management
pub async fn add_shift_required_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    input: web::Json<ShiftRequiredSkillInput>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Manager access required"))
        );
    }

    match skill_repo
        .add_shift_required_skill(input.into_inner())
        .await
    {
        Ok(shift_skill) => Ok(HttpResponse::Created().json(ApiResponse::success(shift_skill))),
        Err(e) => {
            log::error!("Failed to add shift required skill: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to add shift required skill",
                )),
            )
        }
    }
}

pub async fn get_shift_required_skills(
    _user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    path: web::Path<i64>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let shift_id = path.into_inner();

    match skill_repo.get_shift_required_skills(shift_id).await {
        Ok(shift_skills) => Ok(HttpResponse::Ok().json(ApiResponse::success(shift_skills))),
        Err(e) => {
            log::error!("Failed to get shift required skills: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to get shift required skills",
                )),
            )
        }
    }
}

pub async fn remove_shift_required_skill(
    user_context: web::Data<UserContext>,
    skill_repo: web::Data<SkillRepository>,
    path: web::Path<(i64, i64)>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Manager access required"))
        );
    }

    let (shift_id, skill_id) = path.into_inner();

    match skill_repo
        .remove_shift_required_skill(shift_id, skill_id)
        .await
    {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Shift required skill removed successfully",
        ))),
        Ok(false) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::<()>::error("Shift required skill not found"))),
        Err(e) => {
            log::error!("Failed to remove shift required skill: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to remove shift required skill",
                )),
            )
        }
    }
}

// Skill search and matching
pub async fn get_users_with_skill(
    user_context: web::Data<UserContext>,
    _skill_repo: web::Data<SkillRepository>,
    path: web::Path<i64>,
    _query: web::Query<SkillSearchQuery>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if user is admin or manager
    if !user_context.is_manager_or_admin() {
        return Ok(
            HttpResponse::Forbidden().json(ApiResponse::<()>::error("Manager access required"))
        );
    }

    let _skill_id = path.into_inner();

    // Note: Repository method expects Uuid but we have i64 - needs fixing
    log::warn!("Cannot get users with skill due to repository type inconsistency");
    Ok(
        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            "Feature temporarily unavailable due to type mismatch",
        )),
    )
}
