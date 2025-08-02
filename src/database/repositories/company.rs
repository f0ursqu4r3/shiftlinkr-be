use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    models::{
        AddEmployeeToCompanyInput, Company, CompanyEmployee, CompanyEmployeeInfo, CompanyInfo,
        CompanyRole, CreateCompanyInput,
    },
    utils::sql,
};

#[derive(Clone)]
pub struct CompanyRepository {
    pool: PgPool,
}

impl CompanyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_company(&self, request: &CreateCompanyInput) -> Result<Company> {
        let company = sqlx::query_as::<_, Company>(
            r#"
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
                ($1, $2, $3, $4, $5, $6, $7, $8)
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

    pub async fn find_by_id(&self, company_id: Uuid) -> Result<Option<Company>> {
        let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = $1")
            .bind(company_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(company)
    }

    pub async fn find_user_company_info_by_id(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<CompanyInfo>> {
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
        .fetch_optional(&self.pool)
        .await?;

        Ok(company_info)
    }

    pub async fn get_companies_for_user(&self, user_id: Uuid) -> Result<Vec<CompanyInfo>> {
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
                uc.user_id = $1
            ORDER BY
                uc.is_primary DESC,
                c.name ASC
        "#))
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(company_infos)
    }

    pub async fn get_primary_company_for_user(&self, user_id: Uuid) -> Result<Option<CompanyInfo>> {
        let company_info = sqlx::query_as::<_, CompanyInfo>(
            r#"
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
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(company_info)
    }

    pub async fn add_employee_to_company(
        &self,
        company_id: Uuid,
        request: &AddEmployeeToCompanyInput,
    ) -> Result<CompanyEmployee> {
        // If this should be the primary company, unset other primary companies for this user
        if request.is_primary.unwrap_or(false) {
            sqlx::query("UPDATE user_company SET is_primary = false WHERE user_id = $1")
                .bind(request.user_id)
                .execute(&self.pool)
                .await?;
        }

        let company_employee = sqlx::query_as::<_, CompanyEmployee>(
            r#"
            INSERT INTO
                user_company (
                    user_id,
                    company_id,
                    role,
                    is_primary,
                    hire_date
                )
            VALUES
                ($1, $2, $3, $4, $5)
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
            "#,
        )
        .bind(&request.user_id)
        .bind(company_id)
        .bind(&request.role)
        .bind(request.is_primary.unwrap_or(false))
        .bind(&request.hire_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(company_employee)
    }

    pub async fn get_company_employees(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<CompanyEmployeeInfo>> {
        let employess_infos = sqlx::query_as::<_, CompanyEmployeeInfo>(
            r#"
            SELECT
                u.id,
                u.email,
                u.name,
                uc.role,
                uc.is_primary,
                uc.hire_date,
                u.created_at,
                u.updated_at
            FROM
                users u
                JOIN user_company uc ON u.id = uc.user_id
            WHERE
                uc.company_id = $1
            ORDER BY
                uc.role DESC,
                u.name ASC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(employess_infos)
    }

    pub async fn remove_employee_from_company(
        &self,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<()>> {
        let result = sqlx::query("DELETE FROM user_company WHERE company_id = $1 AND user_id = $2")
            .bind(company_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        Ok(Some(()))
    }

    pub async fn update_employee_role(
        &self,
        company_id: Uuid,
        user_id: Uuid,
        role: &CompanyRole,
    ) -> Result<Option<()>> {
        let result = sqlx::query(
            "UPDATE user_company SET role = $1, updated_at = CURRENT_TIMESTAMP WHERE company_id = $2 AND user_id = $3",
        )
        .bind(role)
        .bind(company_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        Ok(Some(()))
    }

    pub async fn check_user_company_access(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<CompanyRole>> {
        let role = sqlx::query_scalar::<_, CompanyRole>(
            "SELECT role FROM user_company WHERE user_id = $1 AND company_id = $2",
        )
        .bind(user_id)
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role)
    }

    pub async fn check_user_company_admin(&self, user_id: Uuid, company_id: Uuid) -> Result<bool> {
        let role = self.check_user_company_access(user_id, company_id).await?;
        Ok(matches!(role, Some(CompanyRole::Admin)))
    }

    pub async fn check_user_company_manager_or_admin(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<bool> {
        let role = self.check_user_company_access(user_id, company_id).await?;
        Ok(matches!(
            role,
            Some(CompanyRole::Admin | CompanyRole::Manager)
        ))
    }

    pub async fn has_primary_company(&self, user_id: Uuid) -> Result<bool> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_company WHERE user_id = $1 AND is_primary = true",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }
}
