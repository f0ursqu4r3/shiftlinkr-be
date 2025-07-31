use actix_web::web::Path;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::config::Config;
use crate::database::models::{
    Action, AddEmployeeToCompanyInput, CompanyInfo, CreateInviteInput, CreateUserInput,
    ForgotPasswordInput, GetInviteResponse, LoginInput, ResetPasswordInput, UserInfo,
};
use crate::database::repositories::invite::InviteRepository;
use crate::database::repositories::UserRepository;
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::repositories::CompanyRepository;
use crate::services::user_context::AsyncUserContext;
use crate::{ActivityLogger, AuthService};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub user: UserInfo,
    pub companies: Vec<CompanyInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteTokenResponse {
    pub invite_link: String,
    pub expires_at: String,
}

pub async fn register(
    auth_service: web::Data<AuthService>,
    request: web::Json<CreateUserInput>,
) -> Result<HttpResponse> {
    let register_request = request.into_inner();

    let response = auth_service.register(register_request).await.map_err(|e| {
        log::error!("Failed to register user: {}", e);
        AppError::DatabaseError(e)
    })?;

    Ok(ApiResponse::success(response))
}

pub async fn login(
    auth_service: web::Data<AuthService>,
    request: web::Json<LoginInput>,
) -> Result<HttpResponse> {
    let login_request = request.into_inner();

    let response = auth_service.login(login_request).await.map_err(|e| {
        log::error!("Failed to login user: {}", e);
        AppError::DatabaseError(e)
    })?;

    Ok(ApiResponse::success(response))
}

pub async fn me(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repository: web::Data<CompanyRepository>,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();
    // Get the user's information
    let user_info = UserInfo::from(user_context.user.clone());

    // Get user's companies
    let companies = company_repository
        .get_companies_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get companies for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    let response = MeResponse {
        user: user_info,
        companies,
    };

    Ok(ApiResponse::success(response))
}

pub async fn forgot_password(
    auth_service: web::Data<AuthService>,
    config: web::Data<Config>,
    request: web::Json<ForgotPasswordInput>,
) -> Result<HttpResponse> {
    let token = auth_service
        .forgot_password(&request.email)
        .await
        .map_err(|e| {
            log::error!("Failed to send password reset email: {}", e);
            AppError::DatabaseError(e)
        })?;

    if config.environment != "production" {
        return Ok(ApiResponse::success(json!({ "token": token })));
    }

    Ok(ApiResponse::success_message(
        "Password reset email sent successfully. Please check your inbox.",
    ))
}

pub async fn reset_password(
    auth_service: web::Data<AuthService>,
    request: web::Json<ResetPasswordInput>,
) -> Result<HttpResponse> {
    auth_service
        .reset_password(&request.token, &request.new_password)
        .await
        .map_err(|e| {
            log::error!("Failed to reset password: {}", e);
            AppError::DatabaseError(e)
        });

    Ok(ApiResponse::success_message(
        "Password has been reset successfully.",
    ))
}

pub async fn create_invite(
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: web::Data<ActivityLogger>,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    invite_repo: web::Data<InviteRepository>,
    config: web::Data<Config>,
    request: web::Json<CreateInviteInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();

    user_context.requires_manager()?;

    let company_id = user_context.company_id().ok_or_else(|| {
        log::error!("User {} does not belong to any company", user_id);
        AppError::PermissionDenied("User does not belong to any company".to_string())
    })?;

    if let Some(user) = user_repo.find_by_email(&request.email).await.map_err(|e| {
        log::error!("Failed to check existing user: {}", e);
        AppError::DatabaseError(e)
    })? {
        // Check if user is already part of the company
        if company_repo
            .check_user_company_access(user.id, company_id)
            .await
            .is_ok()
        {
            return Err(AppError::BadRequest(
                "User with this email already exists in the company".to_string(),
            )
            .into());
        }
    }

    let invite_token = invite_repo
        .create_invite_token(
            &request.email,
            user_id,
            request.role.clone(),
            company_id,
            request.team_id,
        )
        .await
        .map_err(|e| {
            log::error!("Failed to create invite token: {}", e);
            AppError::DatabaseError(e)
        })?;

    let invite_link = format!(
        "{}/auth/invite/{}",
        config.client_base_url, invite_token.token
    );

    // Log invite creation activity
    let metadata = ActivityLogger::metadata(vec![
        (&"email", request.email.clone()),
        (&"role", request.role.to_string()),
        (
            &"team_id",
            request
                .team_id
                .map_or_else(|| "None".to_string(), |id| id.to_string()),
        ),
        (&"invite_link", invite_link.clone()),
        (&"expires_at", invite_token.expires_at.to_rfc3339()),
        (&"company_id", company_id.to_string()),
        (&"inviter_id", user_id.to_string()),
    ]);

    if let Err(err) = activity_logger
        .log_activity(
            company_id,
            Some(user_id),
            "invite_creation".to_string(),
            "invite".to_string(),
            invite_token.id,
            Action::INVITED.to_string(),
            format!("Invite created for email {}", request.email),
            Some(metadata),
            &req,
        )
        .await
    {
        log::error!("Failed to log invite creation activity: {}", err);
    }

    // TODO: Send invite email to the user
    // This is a placeholder for the actual email sending logic
    // You can use a service like SendGrid, Mailgun, etc. to send the email
    // For now, we will just return the invite link in the response

    let invite_token_response = InviteTokenResponse {
        invite_link,
        expires_at: invite_token.expires_at.to_rfc3339(),
    };

    Ok(ApiResponse::success(invite_token_response))
}

pub async fn get_invite(
    invite_repo: web::Data<InviteRepository>,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let token = path.into_inner();

    let invite_token = invite_repo
        .get_invite_token(&token)
        .await
        .map_err(|e| {
            log::error!("Failed to get invite token: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Invite token not found or expired: {}", token);
            AppError::NotFound("Invite token not found or expired".to_string())
        })?;

    // Check if token is expired
    if invite_token.expires_at < chrono::Utc::now() {
        return Err(AppError::BadRequest("Invite token has expired".to_string()).into());
    }

    // Get inviter name from user repository
    let inviter_name = match user_repo.find_by_id(invite_token.inviter_id).await {
        Ok(Some(user)) => user.name,
        _ => "Unknown".to_string(),
    };

    let company_name = match company_repo.find_by_id(invite_token.company_id).await {
        Ok(Some(company)) => company.name,
        _ => "Unknown".to_string(),
    };

    let invite_response = GetInviteResponse {
        email: invite_token.email,
        role: invite_token.role,
        team_id: invite_token.team_id,
        company_id: invite_token.company_id,
        company_name,
        inviter_name,
        expires_at: invite_token.expires_at,
    };

    Ok(ApiResponse::success(invite_response))
}

pub async fn accept_invite(
    AsyncUserContext(user_context): AsyncUserContext,
    invite_repo: web::Data<InviteRepository>,
    company_repo: web::Data<CompanyRepository>,
    token: Path<String>,
) -> Result<HttpResponse> {
    let user = user_context.user;

    // Get invite token
    let invite_token = invite_repo
        .get_invite_token(&token)
        .await
        .map_err(|err| {
            log::error!("Failed to get invite token: {}", err);
            AppError::DatabaseError(err)
        })?
        .ok_or_else(|| {
            log::warn!("Invite token not found or expired: {}", token);
            AppError::NotFound("Invite token not found or expired".to_string())
        })?;

    // Check if token is expired
    if invite_token.expires_at < chrono::Utc::now() {
        return Err(AppError::BadRequest("Invite token has expired".to_string()).into());
    }

    // Check if the user accepting the invite is the same as the user in the token
    if user.email != invite_token.email {
        return Err(AppError::Forbidden("You cannot accept this invite".to_string()).into());
    }

    // Check if the user is already part of the company
    if let Some(_) = company_repo
        .check_user_company_access(user.id, invite_token.company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to check user company access: {}", e);
            AppError::DatabaseError(e)
        })?
    {
        // User is already part of the company
        log::warn!(
            "User {} is already part of company {} when accepting invite",
            user.id,
            invite_token.company_id
        );
        if let Err(err) = invite_repo.mark_invite_token_as_used(&token).await {
            log::error!("Failed to mark invite token as used: {}", err);
        }
        return Err(
            AppError::BadRequest("You are already part of this company".to_string()).into(),
        );
    }

    let has_primary_company = company_repo
        .has_primary_company(user.id)
        .await
        .unwrap_or(false);

    company_repo
        .add_employee_to_company(
            invite_token.company_id,
            &AddEmployeeToCompanyInput {
                user_id: user.id,
                role: Some(invite_token.role),
                is_primary: Some(!has_primary_company), // Set primary company if user has no primary company
                hire_date: None,                        // No hire date provided in invite
            },
        )
        .await
        .map_err(|e| {
            log::error!("Failed to add user to company: {}", e);
            AppError::DatabaseError(e)
        })?;

    // Mark invite as used
    if let Err(err) = invite_repo.mark_invite_token_as_used(&token).await {
        log::error!("Failed to mark invite token as used: {}", err);
    };
    // Log successful invite acceptance
    // Return an Ok response with user info
    let user_info = UserInfo::from(user.clone());

    // Get user's companies
    let companies = company_repo
        .get_companies_for_user(user.id)
        .await
        .map_err(|e| {
            log::error!("Failed to get companies for user {}: {}", user.id, e);
            AppError::DatabaseError(e)
        })?;

    let response = MeResponse {
        user: user_info,
        companies,
    };

    Ok(ApiResponse::success(response))
}

pub async fn reject_invite(
    AsyncUserContext(user_context): AsyncUserContext,
    user_repo: web::Data<UserRepository>,
    invite_repo: web::Data<InviteRepository>,
    token: Path<String>,
) -> Result<HttpResponse> {
    // Get invite token
    let invite_token = invite_repo
        .get_invite_token(&token)
        .await
        .map_err(|e| {
            log::error!("Failed to get invite token: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Invite token not found or expired: {}", token);
            AppError::NotFound("Invite token not found or expired".to_string())
        })?;

    let user = user_repo
        .find_by_email(&invite_token.email)
        .await
        .map_err(|e| {
            log::error!("Failed to find user by email {}: {}", invite_token.email, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("User not found for email: {}", invite_token.email);
            AppError::NotFound("User not found".to_string())
        })?;

    // Check if the user rejecting the invite is the same as the user in the token
    user_context.requires_same_user(user.id)?;

    // Mark invite as used
    if let Err(err) = invite_repo.mark_invite_token_as_used(&token).await {
        log::error!("Failed to mark invite token as used: {}", err);
    }

    Ok(ApiResponse::success("Invite rejected successfully"))
}

pub async fn get_my_invites(
    AsyncUserContext(user_context): AsyncUserContext,
    invite_repo: web::Data<InviteRepository>,
) -> Result<HttpResponse> {
    // Extract token from Authorization header
    let user_id = user_context.user_id();

    user_context.requires_manager()?;

    let company_id = user_context.company_id().ok_or_else(|| {
        log::error!("User {} does not belong to any company", user_id);
        AppError::PermissionDenied("User does not belong to any company".to_string())
    })?;

    let invites = invite_repo
        .get_invites_by_inviter(user_id, company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get invites for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(invites))
}

pub async fn switch_company(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repo: web::Data<CompanyRepository>,
    auth_service: web::Data<AuthService>,
    path: Path<Uuid>,
) -> Result<HttpResponse> {
    let new_company_id = path.into_inner();
    let user_id = user_context.user_id();

    // Check if user belongs to the new company
    company_repo
        .check_user_company_access(user_id, new_company_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to check user company access for user {}: {}",
                user_id,
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            AppError::PermissionDenied("You don't have access to this company".to_string())
        })?;

    let response = auth_service
        .switch_company(user_id, new_company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to switch company for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(response))
}
