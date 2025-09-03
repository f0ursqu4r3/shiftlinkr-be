use actix_web::{
    HttpResponse, Result,
    web::{Data, Json, Path},
};
use uuid::Uuid;

use crate::{
    database::{
        models::{
            AddEmployeeToCompanyInput, CompanyInfo, CompanyRole, CreateCompanyInput,
            activity::Action,
        },
        repositories::company as company_repo,
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::{CacheLayer, cache::InvalidationContext, request_info::RequestInfo},
    services::activity_logger,
    user_context::UserContext,
};

pub async fn get_user_companies(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();
    let companies = company_repo::get_companies_for_user(user_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get companies for user {}: {}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    Ok(ApiResponse::success(companies))
}

pub async fn get_user_primary_company(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();
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
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let company_name = request.name.clone();
    let companies = company_repo::get_companies_for_user(user_id)
        .await
        .unwrap_or_default();

    let company = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Create the company
            let company = company_repo::create_company(tx, &request).await?;

            // Add the creator as an admin
            let add_employee_request = AddEmployeeToCompanyInput {
                user_id,
                role: Some(CompanyRole::Admin),
                is_primary: Some(companies.is_empty()), // Make this their primary company if they don't have one
                hire_date: None,
            };

            company_repo::add_employee_to_company(tx, company.id, &add_employee_request).await?;

            // Log company creation activity
            let metadata = activity_logger::metadata(vec![
                (&"company_id".to_string(), company.id.to_string()),
                (&"company_name".to_string(), company_name.clone()),
                (&"creator_user_id".to_string(), user_id.to_string()),
                (&"creator_name".to_string(), ctx.user.name.to_string()),
                (&"creator_role".to_string(), CompanyRole::Admin.to_string()),
            ]);

            if let Err(e) = activity_logger::log_activity(
                tx,
                company.id,
                Some(user_id),
                "company_management".to_string(),
                "company".to_string(),
                company.id,
                Action::CREATED.to_string(),
                format!("Company '{}' created by user {}", company_name, user_id),
                Some(metadata),
                &req_info,
            )
            .await
            {
                log::warn!("Failed to log company creation activity: {}", e);
            }

            Ok(company)
        })
    })
    .await?;

    // Smart cache invalidation - create_company
    cache
        .invalidate(
            "users",
            &InvalidationContext {
                company_id: Some(company.id),
                user_id: Some(user_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company.id),
                ..Default::default()
            },
        )
        .await;

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

pub async fn get_company_employees(ctx: UserContext) -> Result<HttpResponse> {
    let company_id = ctx.strict_company_id()?;

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
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();
    let employee_user_id = input.user_id;
    let employee_role = input.role.clone();

    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;

    // Check if the user is already an employee
    if company_repo::check_user_company_access(company_id, employee_user_id)
        .await
        .is_ok()
    {
        return Err(AppError::BadRequest(format!(
            "User {} is already an employee of company {}",
            employee_user_id, company_id
        ))
        .into());
    }

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Add the employee to the company
            company_repo::add_employee_to_company(tx, company_id, &input).await?;

            // Log activity
            let metadata = activity_logger::metadata(vec![
                (
                    &"employee_user_id".to_string(),
                    employee_user_id.to_string(),
                ),
                (
                    &"employee_role".to_string(),
                    employee_role
                        .as_ref()
                        .map_or("None".to_string(), |r| r.to_string()),
                ),
            ]);

            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(user_id),
                employee_user_id,
                "add_employee",
                format!(
                    "User {} added to company {} with role {:?}",
                    employee_user_id, company_id, employee_role
                ),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    // Smart cache invalidation - add_employee_to_company
    let company_id = ctx.strict_company_id()?;

    cache
        .invalidate(
            "users",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(employee_user_id),
                ..Default::default()
            },
        )
        .await;

    // Employee addition affects shifts and stats
    cache
        .invalidate(
            "shifts",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success_message(
        "Employee added to company successfully",
    ))
}

pub async fn remove_employee_from_company(
    path: Path<Uuid>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let target_user_id = path.into_inner();

    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Remove the employee from the company
            company_repo::remove_employee_from_company(tx, company_id, target_user_id).await?;

            // Log activity
            let metadata = activity_logger::metadata(vec![
                (&"company_id".to_string(), company_id.to_string()),
                (&"removed_user_id".to_string(), target_user_id.to_string()),
            ]);

            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(user_id),
                target_user_id,
                "remove_employee",
                format!(
                    "User {} removed from company {}",
                    target_user_id, company_id
                ),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    // Smart cache invalidation - remove_employee_from_company
    cache
        .invalidate(
            "users",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(target_user_id),
                ..Default::default()
            },
        )
        .await;

    // Employee removal affects shifts and stats
    cache
        .invalidate(
            "shifts",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success_message(
        "Employee removed from company successfully",
    ))
}

pub async fn update_employee_role(
    path: Path<Uuid>,
    input: Json<CompanyRole>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let target_user_id = path.into_inner();

    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;

    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            company_repo::update_employee_role(tx, company_id, target_user_id, &input)
                .await?
                .ok_or_else(|| {
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

            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(user_id),
                target_user_id,
                &"update_employee_role",
                format!(
                    "User {} updated role in company {}",
                    target_user_id, company_id
                ),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(())
        })
    })
    .await?;

    // Smart cache invalidation - update_employee_role
    cache
        .invalidate(
            "users",
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(target_user_id),
                ..Default::default()
            },
        )
        .await;

    // Role changes affect permissions and stats
    cache
        .invalidate(
            "stats",
            &InvalidationContext {
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success_message(
        "Employee role updated successfully",
    ))
}
