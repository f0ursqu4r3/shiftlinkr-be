use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::{Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    models::{
        PtoBalance, PtoBalanceAdjustmentInput, PtoBalanceHistory, PtoBalanceType,
        PtoBalanceUpdateInput, PtoChangeType,
    },
    utils::sql,
};

macro_rules! update_field {
        ($pool:expr, $field_value:expr, $field_name:literal, $user_id:expr, $company_id:expr, $now:expr) => {
            if let Some(value) = $field_value {
                sqlx::query(&format!(
                    "UPDATE user_company SET {} = $1, updated_at = $2 WHERE user_id = $3 AND company_id = $4",
                    $field_name
                ))
                .bind(value)
                .bind($now)
                .bind($user_id)
                .bind($company_id)
                .execute($pool)
                .await?;
            }
        };
    }

#[derive(Clone)]
pub struct PtoBalanceRepository {
    pool: PgPool,
}

impl PtoBalanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get PTO balance for a user in a specific company
    pub async fn get_balance_for_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<PtoBalance>> {
        let pto_balance = sqlx::query_as::<_, PtoBalance>(
            r#"
            SELECT
                user_id,
                pto_balance_hours,
                sick_balance_hours,
                personal_balance_hours,
                pto_accrual_rate,
                hire_date,
                last_accrual_date
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

        Ok(pto_balance)
    }

    /// Update PTO balance for a user in a specific company
    pub async fn update_balance_for_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
        update: PtoBalanceUpdateInput,
    ) -> Result<PtoBalance> {
        let now = Utc::now();

        // Get current balance first
        let current = self.get_balance_for_company(user_id, company_id).await?;
        if current.is_none() {
            return Err(anyhow::anyhow!("User not found in company"));
        }

        // Execute updates for each field that's provided
        update_field!(
            &self.pool,
            update.pto_balance_hours,
            "pto_balance_hours",
            user_id,
            company_id,
            now
        );
        update_field!(
            &self.pool,
            update.sick_balance_hours,
            "sick_balance_hours",
            user_id,
            company_id,
            now
        );
        update_field!(
            &self.pool,
            update.personal_balance_hours,
            "personal_balance_hours",
            user_id,
            company_id,
            now
        );
        update_field!(
            &self.pool,
            update.pto_accrual_rate,
            "pto_accrual_rate",
            user_id,
            company_id,
            now
        );
        update_field!(
            &self.pool,
            update.hire_date,
            "hire_date",
            user_id,
            company_id,
            now
        );

        // Return updated balance
        self.get_balance_for_company(user_id, company_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated balance"))
    }

    /// Adjust PTO balance and create history record for a specific company
    pub async fn adjust_balance_for_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
        adjustment: PtoBalanceAdjustmentInput,
    ) -> Result<PtoBalanceHistory> {
        let now = Utc::now();

        // Get current balance
        let current_balance = self.get_balance_for_company(user_id, company_id).await?;
        if current_balance.is_none() {
            return Err(anyhow::anyhow!("User not found in company"));
        }
        let current_balance = current_balance.unwrap();

        // Calculate new balance
        let (previous_balance, new_balance, field_name) = match adjustment.balance_type {
            PtoBalanceType::Pto => (
                current_balance.pto_balance_hours,
                current_balance.pto_balance_hours + adjustment.hours_changed,
                "pto_balance_hours",
            ),
            PtoBalanceType::Sick => (
                current_balance.sick_balance_hours,
                current_balance.sick_balance_hours + adjustment.hours_changed,
                "sick_balance_hours",
            ),
            PtoBalanceType::Personal => (
                current_balance.personal_balance_hours,
                current_balance.personal_balance_hours + adjustment.hours_changed,
                "personal_balance_hours",
            ),
        };

        // Prevent negative balances
        if new_balance < 0 {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }

        // Update balance in user_company table
        let query = format!(
            "
            UPDATE
                user_company
            SET
                {} = ?,
                updated_at = ?
            WHERE
                user_id = ?
                AND company_id = ?
            ",
            field_name
        );
        sqlx::query(&sql(&query))
            .bind(new_balance)
            .bind(now)
            .bind(user_id)
            .bind(company_id)
            .execute(&self.pool)
            .await?;

        // Create history record
        let balance_type_str = adjustment.balance_type.to_string();
        let change_type_str = adjustment.change_type.to_string();

        let history_row = sqlx::query_as::<_, PtoBalanceHistory>(&sql(r#"
            INSERT INTO
                pto_balance_history (
                    user_id,
                    company_id,
                    balance_type,
                    change_type,
                    hours_changed,
                    previous_balance,
                    new_balance,
                    description,
                    created_at
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id,
                user_id,
                company_id,
                balance_type,
                change_type,
                hours_changed,
                previous_balance,
                new_balance,
                description,
                related_time_off_id,
                created_at
        "#))
        .bind(user_id)
        .bind(company_id)
        .bind(balance_type_str)
        .bind(change_type_str)
        .bind(adjustment.hours_changed)
        .bind(previous_balance)
        .bind(new_balance)
        .bind(adjustment.description)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(history_row)
    }

    /// Get PTO balance history for a user
    pub async fn get_balance_history(
        &self,
        user_id: Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<PtoBalanceHistory>> {
        let limit = limit.unwrap_or(50);

        let history = sqlx::query_as::<_, PtoBalanceHistory>(
            r#"
            SELECT 
                id,
                user_id,
                balance_type,
                change_type,
                hours_changed,
                previous_balance,
                new_balance,
                description,
                related_time_off_id,
                created_at
            FROM
                pto_balance_history
            WHERE
                user_id = $1
            ORDER BY
                created_at DESC
            LIMIT
                ?
            "#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// Process PTO accrual for a user in a specific company
    pub async fn process_accrual_for_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<PtoBalanceHistory>> {
        let current_balance = self.get_balance_for_company(user_id, company_id).await?;
        if current_balance.is_none() {
            return Err(anyhow::anyhow!("User balance not found for company"));
        }
        let current_balance = current_balance.unwrap();

        // Check if user has accrual rate and hire date
        if current_balance.pto_accrual_rate <= BigDecimal::from(0)
            || current_balance.hire_date.is_none()
        {
            return Ok(None);
        }

        let now = Utc::now();
        let today = now.date_naive();
        let hire_date = current_balance.hire_date.unwrap();
        let last_accrual = current_balance.last_accrual_date.unwrap_or(hire_date);

        // Calculate hours to accrue (simple monthly accrual)
        let months_since_last_accrual = (today.year() - last_accrual.year()) * 12
            + (today.month() as i32 - last_accrual.month() as i32);

        if months_since_last_accrual <= 0 {
            return Ok(None);
        }

        let hours_to_accrue = (current_balance
            .pto_accrual_rate
            .to_string()
            .parse::<f32>()
            .unwrap_or(0.0)
            * months_since_last_accrual as f32) as i32;

        if hours_to_accrue <= 0 {
            return Ok(None);
        }

        // Update balance in user_company table
        let new_balance = current_balance.pto_balance_hours + hours_to_accrue;

        sqlx::query(
            "UPDATE user_company SET pto_balance_hours = $1, last_accrual_date = $2, updated_at = $3 WHERE user_id = $4 AND company_id = $5"
        )
        .bind(new_balance)
        .bind(today)
        .bind(now)
        .bind(user_id)
        .bind(company_id)
        .execute(&self.pool)
        .await?;

        // Create history record
        let balance_type_str = PtoBalanceType::Pto.to_string();
        let change_type_str = PtoChangeType::Accrual.to_string();
        let description = format!("Monthly accrual of {} hours", hours_to_accrue);

        let history_row = sqlx::query_as::<_, PtoBalanceHistory>(
            r#"
            INSERT INTO
                pto_balance_history (
                    user_id,
                    balance_type,
                    change_type,
                    hours_changed,
                    previous_balance,
                    new_balance,
                    description,
                    created_at
                )
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id,
                user_id,
                balance_type,
                change_type,
                hours_changed,
                previous_balance,
                new_balance,
                description,
                related_time_off_id,
                created_at
            "#,
        )
        .bind(user_id)
        .bind(balance_type_str)
        .bind(change_type_str)
        .bind(hours_to_accrue)
        .bind(current_balance.pto_balance_hours)
        .bind(new_balance)
        .bind(description)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(history_row))
    }

    /// Use PTO balance for a time-off request in a specific company
    pub async fn use_balance_for_time_off_for_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
        time_off_id: Uuid,
        balance_type: PtoBalanceType,
        hours_used: i32,
    ) -> Result<PtoBalanceHistory> {
        let now = Utc::now();

        // Get current balance
        let current_balance = self.get_balance_for_company(user_id, company_id).await?;
        if current_balance.is_none() {
            return Err(anyhow::anyhow!("User not found in company"));
        }
        let current_balance = current_balance.unwrap();

        // Calculate new balance
        let (previous_balance, new_balance, field_name) = match balance_type {
            PtoBalanceType::Pto => (
                current_balance.pto_balance_hours,
                current_balance.pto_balance_hours - hours_used,
                "pto_balance_hours",
            ),
            PtoBalanceType::Sick => (
                current_balance.sick_balance_hours,
                current_balance.sick_balance_hours - hours_used,
                "sick_balance_hours",
            ),
            PtoBalanceType::Personal => (
                current_balance.personal_balance_hours,
                current_balance.personal_balance_hours - hours_used,
                "personal_balance_hours",
            ),
        };

        // Check if sufficient balance
        if new_balance < 0 {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }

        // Update balance in user_company table
        let query = format!(
            "UPDATE user_company SET {} = $1, updated_at = $2 WHERE user_id = $3 AND company_id = $4",
            field_name
        );
        sqlx::query(&query)
            .bind(new_balance)
            .bind(now)
            .bind(user_id)
            .bind(company_id)
            .execute(&self.pool)
            .await?;

        // Create history record
        let balance_type_str = balance_type.to_string();
        let change_type_str = PtoChangeType::Usage.to_string();
        let hours_changed = -hours_used;

        let history_row = sqlx::query_as::<_, PtoBalanceHistory>(
            r#"
            INSERT INTO
                pto_balance_history (
                    user_id,
                    balance_type,
                    change_type,
                    hours_changed,
                    previous_balance,
                    new_balance,
                    description,
                    related_time_off_id,
                    created_at
                )
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING 
                id,
                user_id,
                balance_type,
                change_type,
                hours_changed,
                previous_balance,
                new_balance,
                description,
                related_time_off_id,
                created_at
            "#,
        )
        .bind(user_id)
        .bind(balance_type_str)
        .bind(change_type_str)
        .bind(hours_changed)
        .bind(previous_balance)
        .bind(new_balance)
        .bind("Time-off request usage")
        .bind(time_off_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(history_row)
    }
}
