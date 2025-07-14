use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::collections::HashMap;

use crate::config::Config;
use crate::database::models::{
    AcceptInviteRequest, Action, AuthResponse, CreateInviteRequest, CreateUserRequest,
    ForgotPasswordRequest, LoginRequest, ResetPasswordRequest,
};
use crate::database::repositories::invite::InviteRepository;
use crate::AppState;

pub async fn register(
    data: web::Data<AppState>,
    request: web::Json<CreateUserRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let register_request = request.into_inner();
    let email = register_request.email.clone();

    match data.auth_service.register(register_request).await {
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
                serde_json::Value::String(user.id.clone()),
            );

            if let Err(e) = data
                .activity_logger
                .log_auth_activity(
                    1,    // Default company for registration
                    None, // Don't pass user_id to avoid foreign key constraint
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

            if let Err(e) = data
                .activity_logger
                .log_auth_activity(
                    1, // Default company for failed registration attempts
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
    data: web::Data<AppState>,
    request: web::Json<LoginRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let login_request = request.into_inner();
    let email = login_request.email.clone();

    match data.auth_service.login(login_request).await {
        Ok(response) => {
            // Log successful login activity
            let user = &response.user;

            // Get user's primary company for logging
            if let Ok(Some(company)) = data
                .company_repository
                .get_primary_company_for_user(&user.id)
                .await
            {
                let mut metadata = HashMap::new();
                metadata.insert("email".to_string(), serde_json::Value::String(email));
                metadata.insert("success".to_string(), serde_json::Value::Bool(true));

                if let Err(e) = data
                    .activity_logger
                    .log_auth_activity(
                        company.id,
                        Some(user.id.parse().unwrap_or(0)),
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
            if let Err(e) = data
                .activity_logger
                .log_auth_activity(
                    1,    // Default company for failed login attempts
                    None, // No user_id for failed attempts
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

pub async fn me(data: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
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
    match data.auth_service.get_user_from_token(&token).await {
        Ok(user) => {
            // Get user's companies
            let companies = match data
                .company_repository
                .get_companies_for_user(&user.id)
                .await
            {
                Ok(companies) => companies,
                Err(_) => vec![], // If error getting companies, return empty list
            };

            // Get primary company
            let primary_company = match data
                .company_repository
                .get_primary_company_for_user(&user.id)
                .await
            {
                Ok(company) => company,
                Err(_) => None,
            };

            Ok(HttpResponse::Ok().json(json!({
                "user": {
                    "id": user.id,
                    "email": user.email,
                    "name": user.name,
                    // TODO: Add role from company_employees table based on selected company
                },
                "companies": companies,
                "primary_company": primary_company
            })))
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
    data: web::Data<AppState>,
    config: web::Data<Config>,
    request: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse> {
    match data.auth_service.forgot_password(&request.email).await {
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
    data: web::Data<AppState>,
    request: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse> {
    match data
        .auth_service
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
    data: web::Data<AppState>,
    invite_repo: web::Data<InviteRepository>,
    request: web::Json<CreateInviteRequest>,
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
    let user = match data.auth_service.get_user_from_token(&token).await {
        Ok(user) => user,
        Err(err) => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": err.to_string()
            })));
        }
    };

    // TODO: Check if user has permission to create invites (admin or manager) based on company-specific role
    // Since roles are now company-specific, we need to check the role in the context of a specific company
    // For now, allowing all authenticated users to create invites
    /*
    match user.role {
        crate::database::models::UserRole::Admin | crate::database::models::UserRole::Manager => {}
        _ => {
            return Ok(HttpResponse::Forbidden().json(json!({
                "error": "You don't have permission to create invites"
            })));
        }
    }
    */

    // Check if email already exists
    match data.auth_service.get_user_by_email(&request.email).await {
        Ok(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "User with this email already exists"
            })));
        }
        Err(_) => {} // User doesn't exist, which is what we want
    }

    match invite_repo
        .create_invite_token(
            &request.email,
            &user.id,
            request.role.clone(),
            request.team_id,
        )
        .await
    {
        Ok(invite_token) => {
            let invite_link = format!("http://localhost:3000/auth/invite/{}", invite_token.token);
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
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let token = path.into_inner();

    match invite_repo.get_invite_token(&token).await {
        Ok(Some(invite_token)) => {
            // Check if token is expired
            if invite_token.expires_at < chrono::Utc::now().naive_utc() {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "Invite token has expired"
                })));
            }

            // Get inviter name
            let inviter_name = "Unknown"; // TODO: Get from user repo

            Ok(HttpResponse::Ok().json(json!({
                "email": invite_token.email,
                "role": invite_token.role,
                "team_id": invite_token.team_id,
                "inviter_name": inviter_name,
                "expires_at": invite_token.expires_at
            })))
        }
        Ok(None) => Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid or expired invite token"
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get invite: {}", err)
        }))),
    }
}

pub async fn accept_invite(
    data: web::Data<AppState>,
    invite_repo: web::Data<InviteRepository>,
    request: web::Json<AcceptInviteRequest>,
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
    if invite_token.expires_at < chrono::Utc::now().naive_utc() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invite token has expired"
        })));
    }

    // Check if user already exists
    match data
        .auth_service
        .get_user_by_email(&invite_token.email)
        .await
    {
        Ok(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "User with this email already exists"
            })));
        }
        Err(_) => {} // User doesn't exist, which is what we want
    }

    // Create user account
    let create_user_request = CreateUserRequest {
        email: invite_token.email.clone(),
        password: request.password.clone(),
        name: request.name.clone(),
        // TODO: Role will be assigned when creating company_employees relationship
    };

    match data.auth_service.register(create_user_request).await {
        Ok(auth_response) => {
            // Mark invite as used
            if let Err(err) = invite_repo.mark_invite_token_as_used(&request.token).await {
                // Log error but don't fail the request since user was created successfully
                eprintln!("Failed to mark invite token as used: {}", err);
            }

            Ok(HttpResponse::Ok().json(auth_response))
        }
        Err(err) => Ok(HttpResponse::BadRequest().json(json!({
            "error": format!("Failed to create user: {}", err)
        }))),
    }
}

pub async fn get_my_invites(
    invite_repo: web::Data<InviteRepository>,
    req: HttpRequest,
    data: web::Data<AppState>,
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
    let user = match data.auth_service.get_user_from_token(&token).await {
        Ok(user) => user,
        Err(err) => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": err.to_string()
            })));
        }
    };

    // TODO: Check if user has permission to view invites (admin or manager) based on company-specific role
    // Since roles are now company-specific, we need to check the role in the context of a specific company
    // For now, allowing all authenticated users to view invites
    /*
    match user.role {
        crate::database::models::UserRole::Admin | crate::database::models::UserRole::Manager => {}
        _ => {
            return Ok(HttpResponse::Forbidden().json(json!({
                "error": "You don't have permission to view invites"
            })));
        }
    }
    */

    match invite_repo.get_invites_by_inviter(&user.id).await {
        Ok(invites) => Ok(HttpResponse::Ok().json(json!({
            "invites": invites
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get invites: {}", err)
        }))),
    }
}
