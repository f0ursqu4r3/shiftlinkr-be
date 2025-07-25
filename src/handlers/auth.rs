use actix_web::web::Path;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::Config;
use crate::database::models::{
    AcceptInviteInput, Action, AddEmployeeToCompanyInput, CompanyInfo, CompanyRole,
    CreateInviteInput, CreateUserInput, ForgotPasswordInput, GetInviteResponse, LoginInput,
    ResetPasswordInput, UserInfo,
};
use crate::database::repositories::invite::InviteRepository;
use crate::database::repositories::UserRepository;
use crate::repositories::CompanyRepository;
use crate::services::user_context::AsyncUserContext;
use crate::{ActivityLogger, AuthService};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub user: UserInfo,
    pub companies: Vec<CompanyInfo>,
}

pub async fn register(
    auth_service: web::Data<AuthService>,
    activity_logger: web::Data<ActivityLogger>,
    request: web::Json<CreateUserInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let register_request = request.into_inner();
    let email = register_request.email.clone();

    match auth_service.register(register_request).await {
        Ok(response) => {
            // Log successful registration activity
            let user = &response.user;

            // For new registrations, we'll log without user_id since they may not be assigned to a company yet
            let mut metadata = HashMap::new();
            metadata.insert("email".to_string(), serde_json::Value::String(email));
            metadata.insert(
                "name".to_string(),
                serde_json::Value::String(user.name.clone()),
            );
            metadata.insert(
                "user_id".to_string(),
                serde_json::Value::String(user.id.to_string()),
            );

            if let Err(e) = activity_logger
                .log_auth_activity(
                    Uuid::nil(), // Default company for registration
                    None,        // Don't pass user_id to avoid foreign key constraint
                    "register",
                    format!("User {} registered successfully", user.email),
                    Some(metadata),
                    &req,
                )
                .await
            {
                log::warn!("Failed to log registration activity: {}", e);
            }

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            // Log failed registration attempt
            let mut metadata = HashMap::new();
            metadata.insert(
                "email".to_string(),
                serde_json::Value::String(email.clone()),
            );
            metadata.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );

            if let Err(e) = activity_logger
                .log_auth_activity(
                    Uuid::nil(), // Default company for failed registration attempts
                    None,
                    "register_failed",
                    format!("Failed registration attempt for email: {}", email),
                    Some(metadata),
                    &req,
                )
                .await
            {
                log::warn!("Failed to log failed registration activity: {}", e);
            }

            Ok(HttpResponse::BadRequest().json(json!({
                "error": err.to_string()
            })))
        }
    }
}

pub async fn login(
    auth_service: web::Data<AuthService>,
    company_repo: web::Data<CompanyRepository>,
    activity_logger: web::Data<ActivityLogger>,
    request: web::Json<LoginInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let login_request = request.into_inner();
    let email = login_request.email.clone();

    match auth_service.login(login_request).await {
        Ok(response) => {
            // Log successful login activity
            let user = &response.user;

            // Get user's primary company for logging
            if let Ok(Some(company)) = company_repo.get_primary_company_for_user(user.id).await {
                let mut metadata = HashMap::new();
                metadata.insert("email".to_string(), serde_json::Value::String(email));
                metadata.insert("success".to_string(), serde_json::Value::Bool(true));

                if let Err(e) = activity_logger
                    .log_auth_activity(
                        company.id,
                        Some(user.id),
                        Action::LOGIN,
                        format!("User {} logged in successfully", user.email),
                        Some(metadata),
                        &req,
                    )
                    .await
                {
                    log::warn!("Failed to log login activity: {}", e);
                }
            }

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            // Log failed login attempt
            let mut metadata = HashMap::new();
            metadata.insert(
                "email".to_string(),
                serde_json::Value::String(email.clone()),
            );
            metadata.insert("success".to_string(), serde_json::Value::Bool(false));
            metadata.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );

            // For failed logins, we'll use company_id = 1 (default company) since we don't know which company
            if let Err(e) = activity_logger
                .log_auth_activity(
                    Uuid::nil(), // Default company for failed login attempts
                    None,        // No user_id for failed attempts
                    "login_failed",
                    format!("Failed login attempt for email: {}", email),
                    Some(metadata),
                    &req,
                )
                .await
            {
                log::warn!("Failed to log failed login activity: {}", e);
            }

            Ok(HttpResponse::BadRequest().json(json!({
                "error": err.to_string()
            })))
        }
    }
}

pub async fn me(
    auth_service: web::Data<AuthService>,
    company_repository: web::Data<CompanyRepository>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Extract token from Authorization header
    let token = match extract_token_from_header(&req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Missing or invalid authorization header"
            })));
        }
    };

    // Verify token and get user
    match auth_service.get_user_from_token(&token).await {
        Ok(user) => {
            // Get the user's information
            let user_info = UserInfo::from(user.clone());

            // Get user's companies
            let companies = match company_repository.get_companies_for_user(user.id).await {
                Ok(companies) => companies,
                Err(_) => vec![], // If error getting companies, return empty list
            };

            let response = MeResponse {
                user: user_info,
                companies,
            };

            Ok(HttpResponse::Ok().json(json!(response)))
        }
        Err(err) => Ok(HttpResponse::Unauthorized().json(json!({
            "error": err.to_string()
        }))),
    }
}

fn extract_token_from_header(req: &HttpRequest) -> Option<String> {
    let auth_header = req.headers().get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].to_string())
    } else {
        None
    }
}

pub async fn forgot_password(
    auth_service: web::Data<AuthService>,
    config: web::Data<Config>,
    request: web::Json<ForgotPasswordInput>,
) -> Result<HttpResponse> {
    match auth_service.forgot_password(&request.email).await {
        Ok(token) => {
            let mut response = serde_json::json!({
                "message": "If the email exists, a password reset link has been sent."
            });

            // Return token in development/test mode for testing purposes
            if config.environment == "development" || config.environment == "test" {
                response["token"] = serde_json::Value::String(token);
            }

            Ok(HttpResponse::Ok().json(response))
        }
        Err(_) => {
            // Don't reveal whether the email exists or not for security
            Ok(HttpResponse::Ok().json(json!({
                "message": "If the email exists, a password reset link has been sent."
            })))
        }
    }
}

pub async fn reset_password(
    auth_service: web::Data<AuthService>,
    request: web::Json<ResetPasswordInput>,
) -> Result<HttpResponse> {
    match auth_service
        .reset_password(&request.token, &request.new_password)
        .await
    {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({
            "message": "Password has been reset successfully."
        }))),
        Err(err) => Ok(HttpResponse::BadRequest().json(json!({
            "error": err.to_string()
        }))),
    }
}

pub async fn create_invite(
    AsyncUserContext(user_context): AsyncUserContext,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    invite_repo: web::Data<InviteRepository>,
    config: web::Data<Config>,
    request: web::Json<CreateInviteInput>,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();
    let company_id =
        user_context.company.as_ref().map(|c| c.id).ok_or_else(|| {
            actix_web::error::ErrorBadRequest("User does not belong to any company")
        })?;

    // Check if user has permission to create invites (admin or manager)
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "You don't have permission to create invites. Only admins and managers can create invites."
        })));
    }

    // Check if email already exists and is already part of the company
    match user_repo.find_by_email(&request.email).await {
        Ok(Some(existing_user)) => {
            // Check if user is already part of the company
            let companies = company_repo.get_companies_for_user(existing_user.id).await;
            if let Ok(companies_vec) = companies {
                if companies_vec.iter().any(|c| c.id == company_id) {
                    return Ok(HttpResponse::BadRequest().json(json!({
                        "error": "User with this email already exists in the company"
                    })));
                }
            }
        }
        Ok(None) => {}
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to check existing user: {}", err)
            })));
        }
    }

    match invite_repo
        .create_invite_token(
            &request.email,
            user_id,
            request.role.clone(),
            company_id,
            request.team_id,
        )
        .await
    {
        Ok(invite_token) => {
            let invite_link = format!("{}/auth/invite/{}", config.base_url, invite_token.token);
            Ok(HttpResponse::Ok().json(json!({
                "invite_link": invite_link,
                "expires_at": invite_token.expires_at
            })))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to create invite: {}", err)
        }))),
    }
}

pub async fn get_invite(
    invite_repo: web::Data<InviteRepository>,
    user_repo: web::Data<UserRepository>,
    company_repo: web::Data<CompanyRepository>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let token = path.into_inner();

    match invite_repo.get_invite_token(&token).await {
        Ok(Some(invite_token)) => {
            // Check if token is expired
            if invite_token.expires_at < chrono::Utc::now() {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "Invite token has expired"
                })));
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

            Ok(HttpResponse::Ok().json(invite_response))
        }
        Ok(None) => Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid or expired invite token"
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get invite: {}", err),
        }))),
    }
}

pub async fn accept_invite(
    auth_service: web::Data<AuthService>,
    invite_repo: web::Data<InviteRepository>,
    company_repo: web::Data<CompanyRepository>,
    user_repo: web::Data<UserRepository>,
    request: web::Json<AcceptInviteInput>,
) -> Result<HttpResponse> {
    // Get invite token
    let invite_token = match invite_repo.get_invite_token(&request.token).await {
        Ok(Some(invite_token)) => invite_token,
        Ok(None) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid or expired invite token"
            })));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get invite: {}", err)
            })));
        }
    };

    // Check if token is expired
    if invite_token.expires_at < chrono::Utc::now() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invite token has expired"
        })));
    }

    // Check if user already exists
    match user_repo.find_by_email(&invite_token.email).await {
        Ok(Some(user)) => {
            let companies = company_repo.get_companies_for_user(user.id).await;
            // If user is already part of the company, return error
            if let Ok(ref companies_vec) = companies {
                // Check if user is already part of the company
                if companies_vec
                    .iter()
                    .any(|c| c.id == invite_token.company_id)
                {
                    return Ok(HttpResponse::BadRequest().json(json!({
                        "error": "User with this email already exists"
                    })));
                }
            }

            // Determine if this should be the user's primary company
            let is_primary = companies.as_ref().map_or(true, |c| c.is_empty());

            // If user exists but is not part of the company, add them to the company
            if let Err(err) = company_repo
                .add_employee_to_company(
                    invite_token.company_id,
                    &AddEmployeeToCompanyInput {
                        user_id: user.id,
                        role: Some(invite_token.role),
                        is_primary: Some(is_primary),
                        hire_date: None, // No hire date provided in invite
                    },
                )
                .await
            {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to add user to company: {}", err)
                })));
            }

            // Mark invite as used
            if let Err(err) = invite_repo.mark_invite_token_as_used(&request.token).await {
                log::error!("Failed to mark invite token as used: {}", err);
            }

            // Return success response
            return Ok(HttpResponse::Ok().json(json!({
                "message": "Invite accepted successfully",
                "user": user,
            })));
        }
        Ok(None) => {}
        Err(_) => {} // User doesn't exist, which is what we want
    }

    // Create user account
    let create_user_request = CreateUserInput {
        email: invite_token.email.clone(),
        password: request.password.clone(),
        name: request.name.clone(),
    };

    match auth_service.register(create_user_request).await {
        Ok(auth_response) => {
            // Get the company from the inviter
            let inviter_company = match company_repo
                .get_primary_company_for_user(invite_token.inviter_id)
                .await
            {
                Ok(Some(company)) => company,
                _ => {
                    // If we can't get the inviter's company, still mark invite as used but log warning
                    log::warn!(
                        "Could not determine company for inviter {} when accepting invite",
                        invite_token.inviter_id
                    );
                    if let Err(err) = invite_repo.mark_invite_token_as_used(&request.token).await {
                        log::error!("Failed to mark invite token as used: {}", err);
                    }
                    return Ok(HttpResponse::Ok().json(auth_response));
                }
            };

            // Create company_employees relationship with the role from the invite
            if let Err(err) = company_repo
                .add_employee_to_company(
                    inviter_company.id,
                    &AddEmployeeToCompanyInput {
                        user_id: auth_response.user.id,
                        role: Some(invite_token.role),
                        is_primary: Some(true), // Set as primary company
                        hire_date: None,        // No hire date provided in invite
                    },
                )
                .await
            {
                log::error!(
                    "Failed to add user {} to company {}: {}",
                    auth_response.user.id,
                    inviter_company.id,
                    err
                );
                // Continue anyway since user was created successfully
            }

            // Mark invite as used
            if let Err(err) = invite_repo.mark_invite_token_as_used(&request.token).await {
                // Log error but don't fail the request since user was created successfully
                log::error!("Failed to mark invite token as used: {}", err);
            }

            Ok(HttpResponse::Ok().json(auth_response))
        }
        Err(err) => Ok(HttpResponse::BadRequest().json(json!({
            "error": format!("Failed to create user: {}", err)
        }))),
    }
}

pub async fn get_my_invites(
    AsyncUserContext(user_context): AsyncUserContext,
    invite_repo: web::Data<InviteRepository>,
    company_repo: web::Data<CompanyRepository>,
) -> Result<HttpResponse> {
    // Extract token from Authorization header
    let user_id = user_context.user_id();

    // Check if user has permission to view invites (admin or manager) based on company-specific role
    let has_permission = match company_repo.get_companies_for_user(user_id).await {
        Ok(companies) => {
            // Check if user has admin or manager role in any company
            companies
                .iter()
                .any(|company| matches!(company.role, CompanyRole::Admin | CompanyRole::Manager))
        }
        _ => false,
    };

    if !has_permission {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "You don't have permission to view invites. Only admins and managers can view invites."
        })));
    }

    match invite_repo.get_invites_by_inviter(user_id).await {
        Ok(invites) => Ok(HttpResponse::Ok().json(json!({
            "invites": invites
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get invites: {}", err)
        }))),
    }
}

pub async fn switch_company(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repo: web::Data<CompanyRepository>,
    auth_service: web::Data<AuthService>,
    activity_logger: web::Data<ActivityLogger>,
    path: Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let new_company_id = path.into_inner();
    let user_id = user_context.user_id();

    // Check if user belongs to the new company
    match company_repo
        .check_user_company_access(user_id, new_company_id)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Ok(HttpResponse::Forbidden().json(json!({
                "error": "You do not belong to this company"
            })));
        }
        Err(err) => {
            log::error!("Failed to check user company access: {}", err);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to check company access"
            })));
        }
    }

    match auth_service.switch_company(user_id, new_company_id).await {
        Ok(response) => {
            // Log company switch activity
            if let Err(e) = activity_logger
                .log_auth_activity(
                    new_company_id,
                    Some(user_id),
                    Action::SWITCH_COMPANY,
                    format!(
                        "User {} switched to company {}",
                        user_context.user.email.clone(),
                        new_company_id
                    ),
                    None,
                    &req,
                )
                .await
            {
                log::warn!("Failed to log company switch activity: {}", e);
            }

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to switch company: {}", err)
        }))),
    }
}
