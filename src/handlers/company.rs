use actix_web::{
    web::{Json, Path},
    HttpRequest, HttpResponse, Result,
};
use uuid::Uuid;

use crate::database::{
    models::{
        activity::Action, AddEmployeeToCompanyInput, CompanyInfo, CompanyRole, CreateCompanyInput,
    },
    repositories::company as company_repo,
};
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::{activity_logger, user_context::extract_context};

pub async fn get_user_companies(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();
    let companies = company_repo::get_companies_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get companies for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(companies))
}

pub async fn get_user_primary_company(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();
    let company = company_repo::get_primary_company_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get primary company for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;
    Ok(ApiResponse::success(company))
}

pub async fn create_company(
    request: Json<CreateCompanyInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;
    let user_id = user_context.user_id();

    let company_name = request.name.clone();
    let companies = company_repo::get_companies_for_user(user_id)
        .await
        .unwrap_or_default();

    // Create the company
    let company = company_repo::create_company(&request).await.map_err(|e| {
        log::error!("Failed to create company for user {}: {}", user_id, e);
        AppError::DatabaseError(e)
    })?;

    // Add the creator as an admin
    let add_employee_request = AddEmployeeToCompanyInput {
        user_id,
        role: Some(CompanyRole::Admin),
        is_primary: Some(companies.is_empty()), // Make this their primary company if they don't have one
        hire_date: None,
    };

    company_repo::add_employee_to_company(company.id, &add_employee_request)
        .await
        .map_err(|e| {
            log::error!("Failed to add employee to company {}: {}", company.id, e);
            AppError::DatabaseError(e)
        })?;

    // Log company creation activity
    let metadata = activity_logger::metadata(vec![
        (&"company_id".to_string(), company.id.to_string()),
        (&"company_name".to_string(), company_name.clone()),
        (&"creator_user_id".to_string(), user_id.to_string()),
        (
            &"creator_name".to_string(),
            user_context.user.name.to_string(),
        ),
        (
            &"creator_role".to_string(),
            user_context.company.unwrap().role.to_string(),
        ),
    ]);

    if let Err(e) = activity_logger::log_activity(
        company.id,
        Some(user_id),
        "company_management".to_string(),
        "company".to_string(),
        company.id,
        Action::CREATED.to_string(),
        format!("Company '{}' created by user {}", company_name, user_id),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log company creation activity: {}", e);
    }

    // Return the company info with the user's role
    Ok(ApiResponse::created(CompanyInfo {
        id: company.id,
        name: company.name,
        description: company.description,
        website: company.website,
        phone: company.phone,
        email: company.email,
        address: company.address,
        logo_url: company.logo_url,
        timezone: company.timezone,
        role: CompanyRole::Admin,
        is_primary: true,
        hire_date: None,
        created_at: company.created_at,
        updated_at: company.updated_at,
    }))
}

pub async fn get_company_employees(req: HttpRequest) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;
    let company_id = user_context.strict_company_id()?;

    let employees = company_repo::get_company_employees(company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get employees for company {}: {}", company_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(employees))
}

pub async fn add_employee_to_company(
    input: Json<AddEmployeeToCompanyInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;
    let user_id = user_context.user_id();

    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;

    company_repo::add_employee_to_company(company_id, &input)
        .await
        .map_err(|e| {
            log::error!("Failed to add employee to company {}: {}", company_id, e);
            AppError::DatabaseError(e)
        })?;

    // Log activity
    let metadata = activity_logger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"employee_user_id".to_string(), input.user_id.to_string()),
        (
            &"employee_role".to_string(),
            input
                .role
                .as_ref()
                .map_or("None".to_string(), |r| r.to_string()),
        ),
    ]);

    if let Err(e) = activity_logger::log_user_activity(
        company_id,
        Some(user_id),
        input.user_id,
        &"add_employee",
        format!(
            "User {} added to company {} with role {:?}",
            input.user_id, company_id, input.role
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log user activity: {}", e);
    }

    Ok(ApiResponse::success_message(
        "Employee added to company successfully",
    ))
}

pub async fn remove_employee_from_company(
    path: Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;
    let user_id = user_context.user_id();

    let target_user_id = path.into_inner();

    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;

    company_repo::remove_employee_from_company(company_id, target_user_id)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to remove employee {} from company: {}",
                target_user_id,
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!(
                "User {} not found in company {}",
                target_user_id,
                company_id
            );
            AppError::NotFound(format!(
                "User {} not found in company {}",
                target_user_id, company_id
            ))
        })?;

    // Log activity
    let metadata = activity_logger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"removed_user_id".to_string(), target_user_id.to_string()),
    ]);

    if let Err(e) = activity_logger::log_user_activity(
        company_id,
        Some(user_id),
        target_user_id,
        &"remove_employee",
        format!(
            "User {} removed from company {}",
            target_user_id, company_id
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log user activity: {}", e);
    }

    Ok(ApiResponse::success_message(
        "Employee removed from company successfully",
    ))
}

pub async fn update_employee_role(
    path: Path<Uuid>,
    input: Json<CompanyRole>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();

    let target_user_id = path.into_inner();

    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;

    company_repo::update_employee_role(company_id, target_user_id, &input)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to update role for user {} in company {}: {}",
                target_user_id,
                company_id,
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!(
                "User {} not found in company {}",
                target_user_id,
                company_id
            );
            AppError::NotFound(format!(
                "User {} not found in company {}",
                target_user_id, company_id
            ))
        })?;

    // Log activity
    let metadata = activity_logger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"user_id".to_string(), target_user_id.to_string()),
        (&"new_role".to_string(), input.to_string()),
    ]);

    if let Err(e) = activity_logger::log_user_activity(
        company_id,
        Some(user_id),
        target_user_id,
        &"update_employee_role",
        format!(
            "User {} updated role in company {}",
            target_user_id, company_id
        ),
        Some(metadata),
        &req,
    )
    .await
    {
        log::warn!("Failed to log user activity: {}", e);
    }

    Ok(ApiResponse::success_message(
        "Employee role updated successfully",
    ))
}
