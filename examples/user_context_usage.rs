// Example usage of UserContextService in handlers

use crate::extract_user_context;
use crate::services::{UserContext, UserContextService};
use actix_web::{web, HttpRequest, HttpResponse, Result};

// Method 1: Using the service directly
pub async fn example_handler_with_service(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    // Extract user context
    let user_context = user_context_service
        .extract_context(&req)
        .await
        .map_err(|e| actix_web::error::ErrorUnauthorized(format!("Auth error: {}", e)))?;

    // Now you have access to user and company information
    println!(
        "User: {} ({})",
        user_context.user.name, user_context.user.email
    );
    println!("User ID: {}", user_context.user_id());

    if let Some(company) = &user_context.company {
        println!("Company: {}", company.name);
        println!("Role: {:?}", user_context.role);
    }

    // Check permissions
    if user_context.is_admin() {
        println!("User is admin");
    } else if user_context.is_manager() {
        println!("User is manager");
    } else {
        println!("User is employee");
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": user_context.user_id(),
        "email": user_context.user_email(),
        "company_id": user_context.company_id(),
        "company_name": user_context.company_name(),
        "role": user_context.role(),
        "is_admin": user_context.is_admin(),
        "is_manager": user_context.is_manager(),
        "is_employee": user_context.is_employee(),
    })))
}

// Method 2: Using the macro (more concise)
pub async fn example_handler_with_macro(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    // Extract user context using macro
    let user_context = extract_user_context!(user_context_service, &req);

    // Check if user can access a specific resource
    let resource_owner_id = uuid::Uuid::new_v4(); // Example resource owner
    if !user_context.can_access_user_resource(resource_owner_id) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Insufficient permissions to access this resource"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Access granted",
        "user": user_context.user.name,
        "company": user_context.company_name(),
    })))
}

// Method 3: Using the helper function
pub async fn example_handler_with_helper(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    // Extract user context using helper function
    let user_context =
        crate::services::user_context::get_user_context(&user_context_service, &req).await?;

    // Example: Check if user belongs to a specific company
    let target_company_id = uuid::Uuid::new_v4(); // Example company ID
    if !user_context.belongs_to_company(target_company_id) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "User does not belong to the target company"
        })));
    }

    // Example: Admin-only operation
    if !user_context.is_admin() {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Admin access required"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Admin operation successful",
        "user": user_context.user.name,
    })))
}

// Example of checking resource ownership
pub async fn get_user_profile(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    let target_user_id = path.into_inner();

    // Check if user can access this profile
    if !user_context.can_access_user_resource(target_user_id) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Cannot access this user's profile"
        })));
    }

    // Proceed with getting the profile...
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Profile access granted",
        "requesting_user": user_context.user.name,
        "target_user_id": target_user_id,
    })))
}

// Example of company-scoped operations
pub async fn company_operation(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);

    // Ensure user has a company
    let company_id = user_context
        .company_id()
        .ok_or_else(|| actix_web::error::ErrorBadRequest("User must belong to a company"))?;

    // Ensure user has sufficient permissions
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Manager or admin access required"
        })));
    }

    // Perform company-scoped operation...
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Company operation successful",
        "company_id": company_id,
        "company_name": user_context.company_name(),
        "user_role": user_context.role(),
    })))
}
