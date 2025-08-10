use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::database::{
    get_pool,
    models::{
        AddEmployeeToCompanyInput, Company, CompanyEmployee, CompanyEmployeeInfo, CompanyInfo,
        CompanyRole, CreateCompanyInput,
    },
    utils::sql,
};

pub async fn create_company(
    tx: &mut Transaction<'_, Postgres>,
    request: &CreateCompanyInput,
) -> Result<Company, sqlx::Error> {
    let company = sqlx::query_as::<_, Company>(&sql(r#"
        INSERT INTO
            companies (
                name,
                description,
                website,
                phone,
                email,
                address,
                logo_url,
                timezone
            )
        VALUES
            (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING
            id,
            name,
            description,
            website,
            phone,
            email,
            address,
            logo_url,
            timezone,
            created_at,
            updated_at
    "#))
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.website)
    .bind(&request.phone)
    .bind(&request.email)
    .bind(&request.address)
    .bind(&request.logo_url)
    .bind(request.timezone.as_deref().unwrap_or("UTC"))
    .fetch_one(&mut **tx)
    .await?;

    Ok(company)
}

pub async fn find_by_id(company_id: Uuid) -> Result<Option<Company>, sqlx::Error> {
    let company = sqlx::query_as::<_, Company>(&sql(r#"
        SELECT
            *
        FROM
            companies
        WHERE
            id = ?
    "#))
    .bind(company_id)
    .fetch_optional(&get_pool().await)
    .await?;

    Ok(company)
}

pub async fn find_user_company_info_by_id(
    user_id: Uuid,
    company_id: Uuid,
) -> Result<Option<CompanyInfo>, sqlx::Error> {
    let company_info = sqlx::query_as::<_, CompanyInfo>(&sql(r#"
        SELECT
            c.id,
            c.name,
            c.description,
            c.website,
            c.phone,
            c.email,
            c.address,
            c.logo_url,
            c.timezone,
            uc.role,
            uc.is_primary,
            uc.hire_date,
            uc.created_at,
            uc.updated_at
        FROM
            companies c
        JOIN
            user_company uc ON c.id = uc.company_id
        WHERE
            uc.user_id = ?
            AND c.id = ?
    "#))
    .bind(user_id)
    .bind(company_id)
    .fetch_optional(&get_pool().await)
    .await?;

    Ok(company_info)
}

pub async fn get_companies_for_user(user_id: Uuid) -> Result<Vec<CompanyInfo>, sqlx::Error> {
    let company_infos = sqlx::query_as::<_, CompanyInfo>(&sql(r#"
        SELECT
            c.id,
            c.name,
            c.description,
            c.website,
            c.phone,
            c.email,
            c.address,
            c.logo_url,
            c.timezone,
            uc.role,
            uc.is_primary,
            uc.hire_date,
            uc.created_at,
            uc.updated_at
        FROM
            companies c
            JOIN user_company uc ON c.id = uc.company_id
        WHERE
            uc.user_id = ?
        ORDER BY
            uc.is_primary DESC,
            c.name ASC
    "#))
    .bind(user_id)
    .fetch_all(&get_pool().await)
    .await?;

    Ok(company_infos)
}

pub async fn get_primary_company_for_user(
    user_id: Uuid,
) -> Result<Option<CompanyInfo>, sqlx::Error> {
    let company_info = sqlx::query_as::<_, CompanyInfo>(&sql(r#"
        SELECT
            c.id,
            c.name,
            c.description,
            c.website,
            c.phone,
            c.email,
            c.address,
            c.logo_url,
            c.timezone,
            uc.role,
            uc.is_primary
        FROM
            companies c
        JOIN
            user_company uc ON c.id = uc.company_id
        WHERE
            uc.user_id = $1 AND uc.is_primary = true
    "#))
    .bind(user_id)
    .fetch_optional(&get_pool().await)
    .await?;

    Ok(company_info)
}

pub async fn add_employee_to_company(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    request: &AddEmployeeToCompanyInput,
) -> Result<CompanyEmployee, sqlx::Error> {
    // If this should be the primary company, unset other primary companies for this user
    if request.is_primary.unwrap_or(false) {
        sqlx::query(&sql(r#"
            UPDATE user_company
            SET
                is_primary = FALSE
            WHERE
                user_id = ?
        "#))
        .bind(request.user_id)
        .execute(&mut **tx)
        .await?;
    }

    let company_employee = sqlx::query_as::<_, CompanyEmployee>(&sql(r#"
        INSERT INTO
            user_company (
                user_id,
                company_id,
                role,
                is_primary,
                hire_date
            )
        VALUES
            (?, ?, ?, ?, ?)
        RETURNING
            id,
            user_id,
            company_id,
            role,
            is_primary,
            hire_date,
            pto_balance_hours,
            sick_balance_hours,
            personal_balance_hours,
            pto_accrual_rate,
            last_accrual_date,
            created_at,
            updated_at
    "#))
    .bind(&request.user_id)
    .bind(company_id)
    .bind(&request.role)
    .bind(request.is_primary.unwrap_or(false))
    .bind(&request.hire_date)
    .fetch_one(&mut **tx)
    .await?;

    Ok(company_employee)
}

pub async fn get_company_employees(
    company_id: Uuid,
) -> Result<Vec<CompanyEmployeeInfo>, sqlx::Error> {
    let employess_infos = sqlx::query_as::<_, CompanyEmployeeInfo>(&sql(r#"
        SELECT
            u.id,
            u.email,
            u.name,
            uc.role,
            uc.hire_date,
            u.created_at,
            u.updated_at
        FROM
            users u
            JOIN user_company uc ON u.id = uc.user_id
        WHERE
            uc.company_id = ?
        ORDER BY
            uc.role DESC,
            u.name ASC
    "#))
    .bind(company_id)
    .fetch_all(&get_pool().await)
    .await?;

    Ok(employess_infos)
}

pub async fn remove_employee_from_company(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Uuid,
) -> Result<Option<()>, sqlx::Error> {
    let result = sqlx::query(&sql(r#"
        DELETE FROM user_company
        WHERE
            company_id = ?
            AND user_id = ?
    "#))
    .bind(company_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    Ok(Some(()))
}

pub async fn update_employee_role(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Uuid,
    role: &CompanyRole,
) -> Result<Option<()>, sqlx::Error> {
    let result = sqlx::query(&sql(r#"
        UPDATE user_company
        SET
            role = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE
            company_id = ?
            AND user_id = ?
    "#))
    .bind(role.to_string())
    .bind(company_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(if result.rows_affected() > 0 {
        Some(())
    } else {
        None
    })
}
pub async fn check_user_company_access(
    user_id: Uuid,
    company_id: Uuid,
) -> Result<Option<CompanyRole>, sqlx::Error> {
    let role = sqlx::query_scalar::<_, CompanyRole>(&sql(r#"
        SELECT
            role
        FROM
            user_company
        WHERE
            user_id = ?
            AND company_id = ?
    "#))
    .bind(user_id)
    .bind(company_id)
    .fetch_optional(&get_pool().await)
    .await?;

    Ok(role)
}

pub async fn check_user_company_admin(
    user_id: Uuid,
    company_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let role = check_user_company_access(user_id, company_id).await?;
    Ok(matches!(role, Some(CompanyRole::Admin)))
}

pub async fn check_user_company_manager_or_admin(
    user_id: Uuid,
    company_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let role = check_user_company_access(user_id, company_id).await?;
    Ok(matches!(
        role,
        Some(CompanyRole::Admin | CompanyRole::Manager)
    ))
}

pub async fn has_primary_company(user_id: Uuid) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(&sql(r#"
        SELECT
            COUNT(*)
        FROM
            user_company
        WHERE
            user_id = ?
            AND is_primary = TRUE
    "#))
    .bind(user_id)
    .fetch_one(&get_pool().await)
    .await?;

    Ok(count > 0)
}
