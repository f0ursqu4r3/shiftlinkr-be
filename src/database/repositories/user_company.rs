use bigdecimal::BigDecimal;
use sqlx::{Postgres, Transaction};

use crate::database::{
    get_pool,
    models::{CreateUserCompanyInput, UpdateUserCompanyInput, UserCompany},
};

pub async fn create_balance(
    tx: &mut Transaction<'_, Postgres>,
    request: &CreateUserCompanyInput,
) -> Result<UserCompany, sqlx::Error> {
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
    .bind(request.pto_accrual_rate.as_ref().map_or_else(|| BigDecimal::from(0), |v| v.clone()))
    .bind(request.hire_date)
    .fetch_one(&mut **tx)
    .await?;

    Ok(balance)
}

pub async fn get_user_balance_for_company(
    user_id: &str,
    company_id: i64,
) -> Result<Option<UserCompany>, sqlx::Error> {
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
    .fetch_optional(&get_pool().await)
    .await?;

    Ok(balance)
}

pub async fn get_user_balances(user_id: &str) -> Result<Vec<UserCompany>, sqlx::Error> {
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
    .fetch_all(&get_pool().await)
    .await?;

    Ok(balances)
}

pub async fn update_balance(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &str,
    company_id: i64,
    request: &UpdateUserCompanyInput,
) -> Result<UserCompany, sqlx::Error> {
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
            "pto_balance_hours" => query_builder = query_builder.bind(request.pto_balance_hours),
            "sick_balance_hours" => query_builder = query_builder.bind(request.sick_balance_hours),
            "personal_balance_hours" => {
                query_builder = query_builder.bind(request.personal_balance_hours)
            }
            "pto_accrual_rate" => query_builder = query_builder.bind(request.pto_accrual_rate.clone()),
            "hire_date" => query_builder = query_builder.bind(request.hire_date),
            "last_accrual_date" => query_builder = query_builder.bind(request.last_accrual_date),
            _ => {}
        }
    }

    query_builder = query_builder.bind(user_id).bind(company_id);

    query_builder.execute(&mut **tx).await?;

    // Return updated balance
    get_user_balance_for_company(user_id, company_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn delete_balance(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &str,
    company_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_company WHERE user_id = $1 AND company_id = $2")
        .bind(user_id)
        .bind(company_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

pub async fn get_company_balances(company_id: i64) -> Result<Vec<UserCompany>, sqlx::Error> {
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
    .fetch_all(&get_pool().await)
    .await?;

    Ok(balances)
}

pub async fn create_or_update_balance(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &str,
    company_id: i64,
    request: &CreateUserCompanyInput,
) -> Result<UserCompany, sqlx::Error> {
    // Try to get existing balance
    if let Some(_) = get_user_balance_for_company(user_id, company_id).await? {
        // Update existing
        let update_request = UpdateUserCompanyInput {
            role: request.role.clone(),
            is_primary: request.is_primary,
            hire_date: request.hire_date,
            pto_balance_hours: request.pto_balance_hours,
            sick_balance_hours: request.sick_balance_hours,
            personal_balance_hours: request.personal_balance_hours,
            pto_accrual_rate: request.pto_accrual_rate.clone(),
            last_accrual_date: None,
            hourly_rate: request.hourly_rate.clone(),
            overtime_rate_multiplier: request.overtime_rate_multiplier.clone(),
        };
        update_balance(tx, user_id, company_id, &update_request).await
    } else {
        // Create new
        create_balance(tx, request).await
    }
}
