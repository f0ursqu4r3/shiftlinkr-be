use actix_web::{HttpResponse, Result, web, web::Path};
use serde;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    config::config,
    database::{
        models::{
            Action, AddEmployeeToCompanyInput, CompanyInfo, CreateInviteInput, CreateUserInput,
            ForgotPasswordInput, GetInviteResponse, LoginInput, ResetPasswordInput, User,
        },
        repositories::{company as company_repo, invite as invite_repo, user as user_repo},
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::request_info::RequestInfo,
    services::{activity_logger, auth},
    user_context::UserContext,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub user: User,
    pub companies: Vec<CompanyInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteTokenResponse {
    pub invite_link: String,
    pub expires_at: String,
}

pub async fn register(
    request: web::Json<CreateUserInput>,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let register_request = request.into_inner();

    let response = auth::register(register_request).await.map_err(|e| {
        log::error!("Failed to register user: {}", e);
        AppError::internal_server_error_message(e.to_string())
    })?;

    // Invalidate cached auth GETs (e.g., /me)
    cache.bump();
    Ok(ApiResponse::success(response))
}

pub async fn login(
    request: web::Json<LoginInput>,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let login_request = request.into_inner();

    let response = auth::login(login_request).await.map_err(|e| {
        log::error!("Failed to login user: {}", e);
        AppError::from(e)
    })?;

    // Invalidate cache: user context may change
    cache.bump();
    Ok(ApiResponse::success(response))
}

pub async fn me(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();
    // Get the user's information
    let user = ctx.user.clone();

    // Get user's companies
    let companies = company_repo::get_companies_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get companies for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    let response = MeResponse { user, companies };

    Ok(ApiResponse::success(response))
}

pub async fn forgot_password(request: web::Json<ForgotPasswordInput>) -> Result<HttpResponse> {
    let token = auth::forgot_password(&request.email).await.map_err(|e| {
        log::error!("Failed to send password reset email: {}", e);
        AppError::from(e)
    })?;

    if config().environment != "production" {
        return Ok(ApiResponse::success(json!({ "token": token })));
    }

    Ok(ApiResponse::success_message(
        "Password reset email sent successfully. Please check your inbox.",
    ))
}

pub async fn reset_password(
    input: web::Json<ResetPasswordInput>,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    auth::reset_password(&input.token, &input.new_password)
        .await
        .map_err(|e| {
            log::error!("Failed to reset password: {}", e);
            AppError::from(e)
        })?;

    // Invalidate auth-related cached responses
    cache.bump();
    Ok(ApiResponse::success_message(
        "Password has been reset successfully.",
    ))
}

pub async fn create_invite(
    ctx: UserContext,
    input: web::Json<CreateInviteInput>,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    ctx.requires_manager()?;

    let company_id = ctx.company_id().ok_or_else(|| {
        log::error!("User {} does not belong to any company", user_id);
        AppError::PermissionDenied("User does not belong to any company".to_string())
    })?;

    if let Some(user) = user_repo::find_by_email(&input.email).await.map_err(|e| {
        log::error!("Failed to check existing user: {}", e);
        AppError::DatabaseError(e)
    })? {
        // Check if user is already part of the company
        if company_repo::check_user_company_access(user.id, company_id)
            .await
            .is_ok()
        {
            return Err(AppError::BadRequest(
                "User with this email already exists in the company".to_string(),
            )
            .into());
        }
    }

    let (invite_token, invite_link) = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let invite_token = invite_repo::create_invite_token(
                tx,
                &input.email,
                user_id,
                input.role.clone(),
                company_id,
                input.team_id,
            )
            .await?;

            let invite_link = format!(
                "{}/auth/invite/{}",
                config().client_base_url,
                invite_token.token
            );

            // Log invite creation activity
            let metadata = activity_logger::metadata(vec![
                (&"email", input.email.clone()),
                (&"role", input.role.to_string()),
                (
                    &"team_id",
                    input
                        .team_id
                        .map_or_else(|| "None".to_string(), |id| id.to_string()),
                ),
                (&"invite_link", invite_link.clone()),
                (&"expires_at", invite_token.expires_at.to_rfc3339()),
                (&"company_id", company_id.to_string()),
                (&"inviter_id", user_id.to_string()),
            ]);

            activity_logger::log_activity(
                tx,
                company_id,
                Some(user_id),
                "invite_creation".to_string(),
                "invite".to_string(),
                invite_token.id,
                Action::INVITED.to_string(),
                format!("Invite created for email {}", input.email),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok((invite_token, invite_link))
        })
    })
    .await?;

    // TODO: Send invite email to the user
    // This is a placeholder for the actual email sending logic
    // You can use a service like SendGrid, Mailgun, etc. to send the email
    // For now, we will just return the invite link in the response

    let invite_token_response = InviteTokenResponse {
        invite_link,
        expires_at: invite_token.expires_at.to_rfc3339(),
    };

    // Invalidate cached invite lists
    cache.bump();
    Ok(ApiResponse::success(invite_token_response))
}

pub async fn get_invite(path: web::Path<String>) -> Result<HttpResponse> {
    let token = path.into_inner();

    let invite_token = invite_repo::get_invite_token(&token)
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
    let inviter_name = match user_repo::find_by_id(invite_token.inviter_id).await {
        Ok(Some(user)) => user.name,
        _ => "Unknown".to_string(),
    };

    let company_name = match company_repo::find_by_id(invite_token.company_id).await {
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
    token: Path<String>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let user = ctx.user;

    // Get invite token
    let invite_token = invite_repo::get_invite_token(&token)
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

    let accepting_user = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Check if the user is already part of the company
            if let Some(_) =
                company_repo::check_user_company_access(user.id, invite_token.company_id).await?
            {
                // User is already part of the company
                log::warn!(
                    "User {} is already part of company {}",
                    user.id,
                    invite_token.company_id
                );
                invite_repo::mark_invite_token_as_used(tx, &token).await?;
                return Err(AppError::BadRequest(
                    "You are already part of this company".to_string(),
                ));
            }

            let has_primary_company = company_repo::has_primary_company(user.id)
                .await
                .unwrap_or(false);

            company_repo::add_employee_to_company(
                tx,
                invite_token.company_id,
                &AddEmployeeToCompanyInput {
                    user_id: user.id,
                    role: Some(invite_token.role.clone()),
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
            invite_repo::mark_invite_token_as_used(tx, &token).await?;

            // log the activity
            let metadata = activity_logger::metadata(vec![
                (&"email", user.email.clone()),
                (&"role", invite_token.role.to_string()),
                (
                    &"team_id",
                    invite_token
                        .team_id
                        .map_or_else(|| "None".to_string(), |id| id.to_string()),
                ),
                (&"company_id", invite_token.company_id.to_string()),
                (&"inviter_id", invite_token.inviter_id.to_string()),
            ]);

            activity_logger::log_activity(
                tx,
                invite_token.company_id,
                Some(user.id),
                "invite_acceptance".to_string(),
                "invite".to_string(),
                invite_token.id,
                Action::ACCEPTED.to_string(),
                format!("Invite accepted for email {}", user.email),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(user.clone())
        })
    })
    .await?;

    // Log successful invite acceptance
    // Return an Ok response with user info
    let user = accepting_user;

    // Get user's companies
    let companies = company_repo::get_companies_for_user(user.id)
        .await
        .map_err(|e| {
            log::error!("Failed to get companies for user {}: {}", user.id, e);
            AppError::DatabaseError(e)
        })?;

    let response = MeResponse { user, companies };

    // Invalidate cached /me and company lists
    cache.bump();
    Ok(ApiResponse::success(response))
}

pub async fn reject_invite(
    token: Path<String>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    // Get invite token
    let invite_token = invite_repo::get_invite_token(&token)
        .await
        .map_err(|e| {
            log::error!("Failed to get invite token: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("Invite token not found or expired: {}", token);
            AppError::NotFound("Invite token not found or expired".to_string())
        })?;

    let user = user_repo::find_by_email(&invite_token.email)
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
    ctx.requires_same_user(user.id)?;

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Log the rejection activity
            let metadata = activity_logger::metadata(vec![
                (&"email", invite_token.email.clone()),
                (&"role", invite_token.role.to_string()),
                (
                    &"team_id",
                    invite_token
                        .team_id
                        .map_or_else(|| "None".to_string(), |id| id.to_string()),
                ),
                (&"company_id", invite_token.company_id.to_string()),
                (&"inviter_id", invite_token.inviter_id.to_string()),
            ]);

            activity_logger::log_activity(
                tx,
                invite_token.company_id,
                Some(user.id),
                "invite_rejection".to_string(),
                "invite".to_string(),
                invite_token.id,
                Action::REJECTED.to_string(),
                format!("Invite rejected for email {}", invite_token.email),
                Some(metadata),
                &req_info,
            )
            .await?;

            // Mark invite as used
            invite_repo::mark_invite_token_as_used(tx, &token).await?;

            Ok(())
        })
    })
    .await?;

    cache.bump();
    Ok(ApiResponse::success("Invite rejected successfully"))
}

pub async fn get_my_invites(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;

    let invites = invite_repo::get_invites_by_inviter(user_id, company_id)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(invites))
}

pub async fn switch_company(
    path: Path<Uuid>,
    ctx: UserContext,
    cache: web::Data<crate::middleware::CacheLayer>,
) -> Result<HttpResponse> {
    let new_company_id = path.into_inner();
    let user_id = ctx.user_id();

    // Check if user belongs to the new company
    company_repo::check_user_company_access(user_id, new_company_id)
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

    let response = auth::switch_company(user_id, new_company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to switch company for user {}: {}", user_id, e);
            AppError::from(e)
        })?;

    // Invalidate cached /me after switching companies
    cache.bump();
    Ok(ApiResponse::success(response))
}
