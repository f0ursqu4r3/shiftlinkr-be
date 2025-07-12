use crate::database::models::{
    AddEmployeeToCompanyRequest, CompanyInfo, CompanyRole, CreateCompanyRequest,
};
use crate::database::repositories::company_repository::CompanyRepository;
use crate::services::auth::Claims;
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Result,
};

pub async fn get_user_companies(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
) -> Result<HttpResponse> {
    match company_repo.get_companies_for_user(&claims.sub).await {
        Ok(companies) => Ok(HttpResponse::Ok().json(companies)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn get_user_primary_company(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
) -> Result<HttpResponse> {
    match company_repo.get_primary_company_for_user(&claims.sub).await {
        Ok(company) => Ok(HttpResponse::Ok().json(company)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn create_company(
    claims: Claims,
    company_repo: Data<CompanyRepository>,
    request: Json<CreateCompanyRequest>,
) -> Result<HttpResponse> {
    // Create the company
    let company = match company_repo.create_company(&request).await {
        Ok(company) => company,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    // Add the creator as an admin
    let add_employee_request = AddEmployeeToCompanyRequest {
        user_id: claims.sub.clone(),
        role: CompanyRole::Admin,
        is_primary: Some(true), // Make this their primary company if they don't have one
        hire_date: None,
    };

    match company_repo
        .add_employee_to_company(company.id, &add_employee_request)
        .await
    {
        Ok(_) => {
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
    path: Path<i64>,
) -> Result<HttpResponse> {
    let company_id = path.into_inner();

    // Check if user has access to this company
    match company_repo
        .check_user_company_access(&claims.sub, company_id)
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
    path: Path<i64>,
    request: Json<AddEmployeeToCompanyRequest>,
) -> Result<HttpResponse> {
    let company_id = path.into_inner();

    // Check if user is admin of this company
    match company_repo
        .check_user_company_admin(&claims.sub, company_id)
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
    path: Path<(i64, String)>,
) -> Result<HttpResponse> {
    let (company_id, user_id) = path.into_inner();

    // Check if user is admin of this company
    match company_repo
        .check_user_company_admin(&claims.sub, company_id)
        .await
    {
        Ok(true) => {
            match company_repo
                .remove_employee_from_company(company_id, &user_id)
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
    path: Path<(i64, String)>,
    role: Json<CompanyRole>,
) -> Result<HttpResponse> {
    let (company_id, user_id) = path.into_inner();

    // Check if user is admin of this company
    match company_repo
        .check_user_company_admin(&claims.sub, company_id)
        .await
    {
        Ok(true) => {
            match company_repo
                .update_employee_role(company_id, &user_id, &role)
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
