use anyhow::Result;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::database::models::{
    AddEmployeeToCompanyRequest, Company, CompanyEmployee, CompanyEmployeeInfo, CompanyInfo,
    CompanyRole, CreateCompanyRequest,
};

pub struct CompanyRepository {
    pool: SqlitePool,
}

impl CompanyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_company(&self, request: &CreateCompanyRequest) -> Result<Company> {
        let company = sqlx::query_as::<_, Company>(
            r#"
            INSERT INTO companies (name, description, website, phone, email, address, logo_url, timezone)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.website)
        .bind(&request.phone)
        .bind(&request.email)
        .bind(&request.address)
        .bind(&request.logo_url)
        .bind(request.timezone.as_deref().unwrap_or("UTC"))
        .fetch_one(&self.pool)
        .await?;

        Ok(company)
    }

    pub async fn get_company_by_id(&self, company_id: i64) -> Result<Option<Company>> {
        let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?1")
            .bind(company_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(company)
    }

    pub async fn get_companies_for_user(&self, user_id: &str) -> Result<Vec<CompanyInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT c.id, c.name, c.description, c.website, c.phone, c.email, c.address, c.logo_url, c.timezone, ce.role, ce.is_primary
            FROM companies c
            JOIN company_employees ce ON c.id = ce.company_id
            WHERE ce.user_id = ?1
            ORDER BY ce.is_primary DESC, c.name ASC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let companies = rows
            .into_iter()
            .map(|row| {
                let role = CompanyRole::from_str(&row.role).unwrap_or_default();
                CompanyInfo {
                    id: row.id,
                    name: row.name,
                    description: row.description,
                    website: row.website,
                    phone: row.phone,
                    email: row.email,
                    address: row.address,
                    logo_url: row.logo_url,
                    timezone: row.timezone.unwrap_or_else(|| "UTC".to_string()),
                    role,
                    is_primary: row.is_primary,
                }
            })
            .collect();

        Ok(companies)
    }

    pub async fn get_primary_company_for_user(&self, user_id: &str) -> Result<Option<CompanyInfo>> {
        let row = sqlx::query!(
            r#"
            SELECT c.id, c.name, c.description, c.website, c.phone, c.email, c.address, c.logo_url, c.timezone, ce.role, ce.is_primary
            FROM companies c
            JOIN company_employees ce ON c.id = ce.company_id
            WHERE ce.user_id = ?1 AND ce.is_primary = true
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let role = CompanyRole::from_str(&row.role).unwrap_or_default();
            CompanyInfo {
                id: row.id,
                name: row.name,
                description: row.description,
                website: row.website,
                phone: row.phone,
                email: row.email,
                address: row.address,
                logo_url: row.logo_url,
                timezone: row.timezone.unwrap_or_else(|| "UTC".to_string()),
                role,
                is_primary: row.is_primary,
            }
        }))
    }

    pub async fn add_employee_to_company(
        &self,
        company_id: i64,
        request: &AddEmployeeToCompanyRequest,
    ) -> Result<CompanyEmployee> {
        // If this should be the primary company, unset other primary companies for this user
        if request.is_primary.unwrap_or(false) {
            sqlx::query("UPDATE company_employees SET is_primary = false WHERE user_id = ?1")
                .bind(&request.user_id)
                .execute(&self.pool)
                .await?;
        }

        let company_employee = sqlx::query_as::<_, CompanyEmployee>(
            r#"
            INSERT INTO company_employees (user_id, company_id, role, is_primary, hired_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING *
            "#,
        )
        .bind(&request.user_id)
        .bind(company_id)
        .bind(&request.role)
        .bind(request.is_primary.unwrap_or(false))
        .bind(&request.hired_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(company_employee)
    }

    pub async fn get_company_employees(&self, company_id: i64) -> Result<Vec<CompanyEmployeeInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT u.id as "id!", u.email, u.name, ce.role, ce.is_primary, ce.hired_at
            FROM users u
            JOIN company_employees ce ON u.id = ce.user_id
            WHERE ce.company_id = ?1
            ORDER BY ce.role DESC, u.name ASC
            "#,
            company_id
        )
        .fetch_all(&self.pool)
        .await?;

        let employees = rows
            .into_iter()
            .map(|row| {
                let role = CompanyRole::from_str(&row.role).unwrap_or_default();
                CompanyEmployeeInfo {
                    id: row.id,
                    email: row.email,
                    name: row.name,
                    role,
                    is_primary: row.is_primary,
                    hired_at: row.hired_at,
                }
            })
            .collect();

        Ok(employees)
    }

    pub async fn remove_employee_from_company(
        &self,
        company_id: i64,
        user_id: &str,
    ) -> Result<bool> {
        let result =
            sqlx::query("DELETE FROM company_employees WHERE company_id = ?1 AND user_id = ?2")
                .bind(company_id)
                .bind(user_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_employee_role(
        &self,
        company_id: i64,
        user_id: &str,
        role: &CompanyRole,
    ) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE company_employees SET role = ?1, updated_at = CURRENT_TIMESTAMP WHERE company_id = ?2 AND user_id = ?3",
        )
        .bind(role)
        .bind(company_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn check_user_company_access(
        &self,
        user_id: &str,
        company_id: i64,
    ) -> Result<Option<CompanyRole>> {
        let role = sqlx::query_scalar::<_, CompanyRole>(
            "SELECT role FROM company_employees WHERE user_id = ?1 AND company_id = ?2",
        )
        .bind(user_id)
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role)
    }

    pub async fn check_user_company_admin(&self, user_id: &str, company_id: i64) -> Result<bool> {
        let role = self.check_user_company_access(user_id, company_id).await?;
        Ok(matches!(role, Some(CompanyRole::Admin)))
    }

    pub async fn check_user_company_manager_or_admin(
        &self,
        user_id: &str,
        company_id: i64,
    ) -> Result<bool> {
        let role = self.check_user_company_access(user_id, company_id).await?;
        Ok(matches!(
            role,
            Some(CompanyRole::Admin | CompanyRole::Manager)
        ))
    }
}
