use actix_web::{web, HttpResponse, Result, HttpRequest};
use serde_json::json;

use crate::database::models::{CreateUserRequest, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest};
use crate::AppState;

pub async fn register(
    data: web::Data<AppState>,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    match data.auth_service.register(request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(err) => Ok(HttpResponse::BadRequest().json(json!({
            "error": err.to_string()
        }))),
    }
}

pub async fn login(
    data: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    match data.auth_service.login(request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(err) => Ok(HttpResponse::BadRequest().json(json!({
            "error": err.to_string()
        }))),
    }
}

pub async fn me(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Extract token from Authorization header
    let token = match extract_token_from_header(&req) {
        Some(token) => token,
        None => return Ok(HttpResponse::Unauthorized().json(json!({
            "error": "Missing or invalid authorization header"
        }))),
    };

    // Verify token and get user
    match data.auth_service.get_user_from_token(&token).await {
        Ok(user) => Ok(HttpResponse::Ok().json(json!({
            "user": {
                "id": user.id,
                "email": user.email,
                "name": user.name,
                "role": user.role,
            }
        }))),
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
    request: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse> {
    match data.auth_service.forgot_password(&request.email).await {
        Ok(_token) => Ok(HttpResponse::Ok().json(json!({
            "message": "If the email exists, a password reset link has been sent."
        }))),
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
    match data.auth_service.reset_password(&request.token, &request.new_password).await {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({
            "message": "Password has been reset successfully."
        }))),
        Err(err) => Ok(HttpResponse::BadRequest().json(json!({
            "error": err.to_string()
        }))),
    }
}
