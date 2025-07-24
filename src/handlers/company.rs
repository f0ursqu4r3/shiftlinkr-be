use crate::database::models::activity::Action;
use crate::database::models::{
    AddEmployeeToCompanyInput, CompanyInfo, CompanyRole, CreateCompanyInput,
};
use crate::database::repositories::company::CompanyRepository;
use crate::services::activity_logger::ActivityLogger;
use crate::services::auth::Claims;
use crate::services::user_context::{self, AsyncUserContext};
use actix_web::{
    web::{Data, Json, Path},
    HttpRequest, HttpResponse, Result,
};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn get_user_companies(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;
    match company_repo.get_companies_for_user(user_id).await {
        Ok(companies) => Ok(HttpResponse::Ok().json(companies)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn get_user_primary_company(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;
    match company_repo.get_primary_company_for_user(user_id).await {
        Ok(company) => Ok(HttpResponse::Ok().json(company)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
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
    let company = match company_repo.create_company(&request).await {
        Ok(company) => company,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    // Add the creator as an admin
    let add_employee_request = AddEmployeeToCompanyInput {
        user_id,
        role: Some(CompanyRole::Admin),
        is_primary: Some(companies.is_empty()), // Make this their primary company if they don't have one
        hire_date: None,
    };

    match company_repo
        .add_employee_to_company(company.id, &add_employee_request)
        .await
    {
        Ok(_) => {
            // Log company creation activity
            let mut metadata = HashMap::new();
            metadata.insert(
                "company_name".to_string(),
                serde_json::Value::String(company_name.clone()),
            );
            metadata.insert(
                "creator_user_id".to_string(),
                serde_json::Value::String(user_id.to_string()),
            );
            metadata.insert(
                "creator_role".to_string(),
                serde_json::Value::String("Admin".to_string()),
            );

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
            Ok(HttpResponse::Created().json(CompanyInfo {
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
            }))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn get_company_employees(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
    path: Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;

    let company_id = path.into_inner();

    // Check if user has access to this company
    match company_repo
        .check_user_company_access(user_id, company_id)
        .await
    {
        Ok(Some(_)) => {
            // User has access, proceed
            match company_repo.get_company_employees(company_id).await {
                Ok(employees) => Ok(HttpResponse::Ok().json(employees)),
                Err(_) => Ok(HttpResponse::InternalServerError().finish()),
            }
        }
        Ok(None) => Ok(HttpResponse::Forbidden().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn add_employee_to_company(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
    path: Path<Uuid>,
    request: Json<AddEmployeeToCompanyInput>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;

    let company_id = path.into_inner();

    // Check if user is admin of this company
    match company_repo
        .check_user_company_admin(user_id, company_id)
        .await
    {
        Ok(true) => {
            match company_repo
                .add_employee_to_company(company_id, &request)
                .await
            {
                Ok(_) => Ok(HttpResponse::Created().finish()),
                Err(_) => Ok(HttpResponse::InternalServerError().finish()),
            }
        }
        Ok(false) => Ok(HttpResponse::Forbidden().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn remove_employee_from_company(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
    path: Path<(Uuid, Uuid)>,
) -> Result<HttpResponse> {
    let claims_user = claims.sub;

    let (company_id, user_id) = path.into_inner();

    // Check if user is admin of this company
    match company_repo
        .check_user_company_admin(claims_user, company_id)
        .await
    {
        Ok(true) => {
            match company_repo
                .remove_employee_from_company(company_id, user_id)
                .await
            {
                Ok(true) => Ok(HttpResponse::NoContent().finish()),
                Ok(false) => Ok(HttpResponse::NotFound().finish()),
                Err(_) => Ok(HttpResponse::InternalServerError().finish()),
            }
        }
        Ok(false) => Ok(HttpResponse::Forbidden().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn update_employee_role(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
    path: Path<(Uuid, Uuid)>,
    role: Json<CompanyRole>,
) -> Result<HttpResponse> {
    let claims_user = claims.sub;

    let (company_id, user_id) = path.into_inner();

    // Check if user is admin of this company
    match company_repo
        .check_user_company_admin(claims_user, company_id)
        .await
    {
        Ok(true) => {
            match company_repo
                .update_employee_role(company_id, user_id, &role)
                .await
            {
                Ok(true) => Ok(HttpResponse::NoContent().finish()),
                Ok(false) => Ok(HttpResponse::NotFound().finish()),
                Err(_) => Ok(HttpResponse::InternalServerError().finish()),
            }
        }
        Ok(false) => Ok(HttpResponse::Forbidden().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
