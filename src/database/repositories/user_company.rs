use anyhow::Result;
use sqlx::PgPool;

use crate::database::models::{CreateUserCompanyInput, UpdateUserCompanyInput, UserCompany};

#[derive(Clone)]
pub struct UserCompanyRepository {
    pool: PgPool,
}

impl UserCompanyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_balance(&self, request: &CreateUserCompanyInput) -> Result<UserCompany> {
        let balance = sqlx::query_as::<_, UserCompany>(
            r#"
            INSERT INTO
                user_company (
                    user_id,
                    company_id,
                    pto_balance_hours,
                    sick_balance_hours,
                    personal_balance_hours,
                    pto_accrual_rate,
                    hire_date
                )
            VALUES
                ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id,
                user_id,
                company_id,
                pto_balance_hours,
                sick_balance_hours,
                personal_balance_hours,
                pto_accrual_rate,
                hire_date,
                last_accrual_date,
                created_at,
                updated_at
            "#,
        )
        .bind(&request.user_id)
        .bind(&request.company_id)
        .bind(request.pto_balance_hours.unwrap_or(0))
        .bind(request.sick_balance_hours.unwrap_or(0))
        .bind(request.personal_balance_hours.unwrap_or(0))
        .bind(request.pto_accrual_rate.unwrap_or(0.0))
        .bind(request.hire_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(balance)
    }

    pub async fn get_user_balance_for_company(
        &self,
        user_id: &str,
        company_id: i64,
    ) -> Result<Option<UserCompany>> {
        let balance = sqlx::query_as::<_, UserCompany>(
            r#"
            SELECT
                id,
                user_id,
                company_id,
                pto_balance_hours,
                sick_balance_hours,
                personal_balance_hours,
                pto_accrual_rate,
                hire_date,
                last_accrual_date,
                created_at,
                updated_at
            FROM
                user_company
            WHERE
                user_id = $1
                AND company_id = $2
            "#,
        )
        .bind(user_id)
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(balance)
    }

    pub async fn get_user_balances(&self, user_id: &str) -> Result<Vec<UserCompany>> {
        let balances = sqlx::query_as::<_, UserCompany>(
            r#"
            SELECT
                id,
                user_id,
                company_id,
                pto_balance_hours,
                sick_balance_hours,
                personal_balance_hours,
                pto_accrual_rate,
                hire_date,
                last_accrual_date,
                created_at,
                updated_at
            FROM
                user_company
            WHERE
                user_id = $1
            ORDER BY
                company_id
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(balances)
    }

    pub async fn update_balance(
        &self,
        user_id: &str,
        company_id: i64,
        request: &UpdateUserCompanyInput,
    ) -> Result<UserCompany> {
        // Build dynamic update query based on provided fields
        let mut query = "UPDATE user_company SET updated_at = CURRENT_TIMESTAMP".to_string();
        let mut params: Vec<String> = vec![];

        if request.pto_balance_hours.is_some() {
            query.push_str(&format!(", pto_balance_hours = ${}", params.len() + 1));
            params.push("pto_balance_hours".to_string());
        }
        if request.sick_balance_hours.is_some() {
            query.push_str(&format!(", sick_balance_hours = ${}", params.len() + 1));
            params.push("sick_balance_hours".to_string());
        }
        if request.personal_balance_hours.is_some() {
            query.push_str(&format!(", personal_balance_hours = ${}", params.len() + 1));
            params.push("personal_balance_hours".to_string());
        }
        if request.pto_accrual_rate.is_some() {
            query.push_str(&format!(", pto_accrual_rate = ${}", params.len() + 1));
            params.push("pto_accrual_rate".to_string());
        }
        if request.hire_date.is_some() {
            query.push_str(&format!(", hire_date = ${}", params.len() + 1));
            params.push("hire_date".to_string());
        }
        if request.last_accrual_date.is_some() {
            query.push_str(&format!(", last_accrual_date = ${}", params.len() + 1));
            params.push("last_accrual_date".to_string());
        }

        query.push_str(&format!(
            " WHERE user_id = ${} AND company_id = ${}",
            params.len() + 1,
            params.len() + 2
        ));

        let mut query_builder = sqlx::query(&query);

        // Bind parameters in order
        for param in &params {
            match param.as_str() {
                "pto_balance_hours" => {
                    query_builder = query_builder.bind(request.pto_balance_hours)
                }
                "sick_balance_hours" => {
                    query_builder = query_builder.bind(request.sick_balance_hours)
                }
                "personal_balance_hours" => {
                    query_builder = query_builder.bind(request.personal_balance_hours)
                }
                "pto_accrual_rate" => query_builder = query_builder.bind(request.pto_accrual_rate),
                "hire_date" => query_builder = query_builder.bind(request.hire_date),
                "last_accrual_date" => {
                    query_builder = query_builder.bind(request.last_accrual_date)
                }
                _ => {}
            }
        }

        query_builder = query_builder.bind(user_id).bind(company_id);

        query_builder.execute(&self.pool).await?;

        // Return updated balance
        self.get_user_balance_for_company(user_id, company_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Balance not found after update"))
    }

    pub async fn delete_balance(&self, user_id: &str, company_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM user_company WHERE user_id = $1 AND company_id = $2")
            .bind(user_id)
            .bind(company_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_company_balances(&self, company_id: i64) -> Result<Vec<UserCompany>> {
        let balances = sqlx::query_as::<_, UserCompany>(
            r#"
            SELECT
                id,
                user_id,
                company_id,
                pto_balance_hours,
                sick_balance_hours,
                personal_balance_hours,
                pto_accrual_rate,
                hire_date,
                last_accrual_date,
                created_at,
                updated_at
            FROM
                user_company ucb
            WHERE
                ucb.company_id = $1
            ORDER BY
                ucb.user_id
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(balances)
    }

    pub async fn create_or_update_balance(
        &self,
        user_id: &str,
        company_id: i64,
        request: &CreateUserCompanyInput,
    ) -> Result<UserCompany> {
        // Try to get existing balance
        if let Some(_existing) = self
            .get_user_balance_for_company(user_id, company_id)
            .await?
        {
            // Update existing
            let update_request = UpdateUserCompanyInput {
                pto_balance_hours: request.pto_balance_hours,
                sick_balance_hours: request.sick_balance_hours,
                personal_balance_hours: request.personal_balance_hours,
                pto_accrual_rate: request.pto_accrual_rate,
                hire_date: request.hire_date,
                last_accrual_date: None,
            };
            self.update_balance(user_id, company_id, &update_request)
                .await
        } else {
            // Create new
            self.create_balance(request).await
        }
    }
}
