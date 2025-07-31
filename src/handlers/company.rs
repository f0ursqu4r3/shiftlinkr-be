use crate::database::models::activity::Action;
use crate::database::models::{
    AddEmployeeToCompanyInput, CompanyInfo, CompanyRole, CreateCompanyInput,
};
use crate::database::repositories::company::CompanyRepository;
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::activity_logger::ActivityLogger;
use crate::services::user_context::AsyncUserContext;
use actix_web::{
    web::{Data, Json, Path},
    HttpRequest, HttpResponse, Result,
};
use uuid::Uuid;

pub async fn get_user_companies(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repo: Data<CompanyRepository>,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();
    let companies = company_repo
        .get_companies_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get companies for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(companies))
}

pub async fn get_user_primary_company(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repo: Data<CompanyRepository>,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();
    let company = company_repo
        .get_primary_company_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get primary company for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;
    Ok(ApiResponse::success(company))
}

pub async fn create_company(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repo: Data<CompanyRepository>,
    activity_logger: Data<ActivityLogger>,
    request: Json<CreateCompanyInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();

    let company_name = request.name.clone();
    let companies = company_repo
        .get_companies_for_user(user_id)
        .await
        .unwrap_or_default();

    // Create the company
    let company = company_repo.create_company(&request).await.map_err(|e| {
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

    company_repo
        .add_employee_to_company(company.id, &add_employee_request)
        .await
        .map_err(|e| {
            log::error!("Failed to add employee to company {}: {}", company.id, e);
            AppError::DatabaseError(e)
        })?;

    // Log company creation activity
    let metadata = ActivityLogger::metadata(vec![
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

    if let Err(e) = activity_logger
        .log_activity(
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

pub async fn get_company_employees(
    AsyncUserContext(user_context): AsyncUserContext,
    company_repo: Data<CompanyRepository>,
) -> Result<HttpResponse> {
    let company_id = user_context.strict_company_id()?;

    let employees = company_repo
        .get_company_employees(company_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get employees for company {}: {}", company_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(employees))
}

pub async fn add_employee_to_company(
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: Data<ActivityLogger>,
    company_repo: Data<CompanyRepository>,
    request: Json<AddEmployeeToCompanyInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = user_context.user_id();

    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;

    company_repo
        .add_employee_to_company(company_id, &request)
        .await
        .map_err(|e| {
            log::error!("Failed to add employee to company {}: {}", company_id, e);
            AppError::DatabaseError(e)
        })?;

    // Log activity
    let metadata = ActivityLogger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"employee_user_id".to_string(), request.user_id.to_string()),
        (
            &"employee_role".to_string(),
            request
                .role
                .as_ref()
                .map_or("None".to_string(), |r| r.to_string()),
        ),
    ]);

    if let Err(e) = activity_logger
        .log_user_activity(
            company_id,
            Some(user_id),
            request.user_id,
            &"add_employee",
            format!(
                "User {} added to company {} with role {:?}",
                request.user_id, company_id, request.role
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
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: Data<ActivityLogger>,
    company_repo: Data<CompanyRepository>,
    path: Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user = user_context.user_id();

    let user_id = path.into_inner();

    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;

    company_repo
        .remove_employee_from_company(company_id, user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to remove employee {} from company: {}", user_id, e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("User {} not found in company {}", user_id, company_id);
            AppError::NotFound(format!(
                "User {} not found in company {}",
                user_id, company_id
            ))
        })?;

    // Log activity
    let metadata = ActivityLogger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"removed_user_id".to_string(), user_id.to_string()),
    ]);

    if let Err(e) = activity_logger
        .log_user_activity(
            company_id,
            Some(user),
            user_id,
            &"remove_employee",
            format!("User {} removed from company {}", user_id, company_id),
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
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: Data<ActivityLogger>,
    company_repo: Data<CompanyRepository>,
    path: Path<Uuid>,
    role: Json<CompanyRole>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let claims_user = user_context.user_id();

    let user_id = path.into_inner();

    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;

    company_repo
        .update_employee_role(company_id, user_id, &role)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to update role for user {} in company {}: {}",
                user_id,
                company_id,
                e
            );
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| {
            log::warn!("User {} not found in company {}", user_id, company_id);
            AppError::NotFound(format!(
                "User {} not found in company {}",
                user_id, company_id
            ))
        })?;

    // Log activity
    let metadata = ActivityLogger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"user_id".to_string(), user_id.to_string()),
        (&"new_role".to_string(), role.to_string()),
    ]);

    if let Err(e) = activity_logger
        .log_user_activity(
            company_id,
            Some(claims_user),
            user_id,
            &"update_employee_role",
            format!("User {} updated role in company {}", user_id, company_id),
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
