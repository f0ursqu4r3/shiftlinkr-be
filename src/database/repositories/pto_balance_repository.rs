use anyhow::Result;
use chrono::{Datelike, Utc};
use sqlx::SqlitePool;

use crate::database::models::{
    PtoBalance, PtoBalanceAdjustment, PtoBalanceHistory, PtoBalanceType, PtoBalanceUpdate,
    PtoChangeType,
};

#[derive(Clone)]
pub struct PtoBalanceRepository {
    pool: SqlitePool,
}

impl PtoBalanceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get PTO balance for a user
    pub async fn get_balance(&self, user_id: &str) -> Result<Option<PtoBalance>> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id as user_id,
                pto_balance_hours,
                sick_balance_hours,
                personal_balance_hours,
                pto_accrual_rate,
                hire_date,
                last_accrual_date
            FROM users 
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(PtoBalance {
                user_id: row.user_id.unwrap_or_default(),
                pto_balance_hours: row.pto_balance_hours.unwrap_or(0) as i32,
                sick_balance_hours: row.sick_balance_hours.unwrap_or(0) as i32,
                personal_balance_hours: row.personal_balance_hours.unwrap_or(0) as i32,
                pto_accrual_rate: row.pto_accrual_rate.unwrap_or(0.0) as f32,
                hire_date: row.hire_date.map(|d| d.and_hms_opt(0, 0, 0).unwrap()),
                last_accrual_date: row
                    .last_accrual_date
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update PTO balance for a user
    pub async fn update_balance(
        &self,
        user_id: &str,
        update: PtoBalanceUpdate,
    ) -> Result<PtoBalance> {
        let now = Utc::now().naive_utc();

        // Get current balance first
        let current = self.get_balance(user_id).await?;
        if current.is_none() {
            return Err(anyhow::anyhow!("User not found"));
        }

        // Execute updates for each field that's provided
        if let Some(hours) = update.pto_balance_hours {
            sqlx::query("UPDATE users SET pto_balance_hours = ?, updated_at = ? WHERE id = ?")
                .bind(hours)
                .bind(now)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        if let Some(hours) = update.sick_balance_hours {
            sqlx::query("UPDATE users SET sick_balance_hours = ?, updated_at = ? WHERE id = ?")
                .bind(hours)
                .bind(now)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        if let Some(hours) = update.personal_balance_hours {
            sqlx::query("UPDATE users SET personal_balance_hours = ?, updated_at = ? WHERE id = ?")
                .bind(hours)
                .bind(now)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        if let Some(rate) = update.pto_accrual_rate {
            sqlx::query("UPDATE users SET pto_accrual_rate = ?, updated_at = ? WHERE id = ?")
                .bind(rate)
                .bind(now)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        if let Some(hire_date) = update.hire_date {
            sqlx::query("UPDATE users SET hire_date = ?, updated_at = ? WHERE id = ?")
                .bind(hire_date.date())
                .bind(now)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        // Return updated balance
        self.get_balance(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated balance"))
    }

    /// Adjust PTO balance and create history record
    pub async fn adjust_balance(
        &self,
        user_id: &str,
        adjustment: PtoBalanceAdjustment,
    ) -> Result<PtoBalanceHistory> {
        let now = Utc::now().naive_utc();

        // Get current balance
        let current_balance = self.get_balance(user_id).await?;
        if current_balance.is_none() {
            return Err(anyhow::anyhow!("User not found"));
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

        // Update balance in users table
        let query = format!(
            "UPDATE users SET {} = ?, updated_at = ? WHERE id = ?",
            field_name
        );
        sqlx::query(&query)
            .bind(new_balance)
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Create history record
        let balance_type_str = adjustment.balance_type.to_string();
        let change_type_str = PtoChangeType::Adjustment.to_string();

        let history_row = sqlx::query!(
            r#"
            INSERT INTO pto_balance_history (
                user_id, balance_type, change_type, hours_changed, 
                previous_balance, new_balance, description, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, user_id, balance_type, change_type, hours_changed,
                previous_balance, new_balance, description, related_time_off_id, created_at
            "#,
            user_id,
            balance_type_str,
            change_type_str,
            adjustment.hours_changed,
            previous_balance,
            new_balance,
            adjustment.description,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PtoBalanceHistory {
            id: history_row.id,
            user_id: history_row.user_id,
            balance_type: balance_type_str
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?,
            change_type: change_type_str
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?,
            hours_changed: history_row.hours_changed as i32,
            previous_balance: history_row.previous_balance as i32,
            new_balance: history_row.new_balance as i32,
            description: history_row.description,
            related_time_off_id: history_row.related_time_off_id,
            created_at: history_row.created_at.unwrap_or(now),
        })
    }

    /// Get PTO balance history for a user
    pub async fn get_balance_history(
        &self,
        user_id: &str,
        limit: Option<i32>,
    ) -> Result<Vec<PtoBalanceHistory>> {
        let limit = limit.unwrap_or(50);

        let rows = sqlx::query!(
            r#"
            SELECT 
                id, user_id, balance_type, change_type, hours_changed,
                previous_balance, new_balance, description, related_time_off_id, created_at
            FROM pto_balance_history 
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut history = Vec::new();
        for row in rows {
            history.push(PtoBalanceHistory {
                id: row.id.unwrap_or(0),
                user_id: row.user_id,
                balance_type: row
                    .balance_type
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?,
                change_type: row
                    .change_type
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?,
                hours_changed: row.hours_changed as i32,
                previous_balance: row.previous_balance as i32,
                new_balance: row.new_balance as i32,
                description: row.description,
                related_time_off_id: row.related_time_off_id,
                created_at: row.created_at.unwrap_or_else(|| Utc::now().naive_utc()),
            });
        }

        Ok(history)
    }

    /// Process PTO accrual for a user
    pub async fn process_accrual(&self, user_id: &str) -> Result<Option<PtoBalanceHistory>> {
        let current_balance = self.get_balance(user_id).await?;
        if current_balance.is_none() {
            return Err(anyhow::anyhow!("User not found"));
        }
        let current_balance = current_balance.unwrap();

        // Check if user has accrual rate and hire date
        if current_balance.pto_accrual_rate <= 0.0 || current_balance.hire_date.is_none() {
            return Ok(None);
        }

        let now = Utc::now().naive_utc();
        let hire_date = current_balance.hire_date.unwrap();
        let last_accrual = current_balance.last_accrual_date.unwrap_or(hire_date);

        // Calculate hours to accrue (simple monthly accrual)
        let months_since_last_accrual = (now.year() - last_accrual.year()) * 12
            + (now.month() as i32 - last_accrual.month() as i32);

        if months_since_last_accrual <= 0 {
            return Ok(None);
        }

        let hours_to_accrue =
            (current_balance.pto_accrual_rate * months_since_last_accrual as f32) as i32;

        if hours_to_accrue <= 0 {
            return Ok(None);
        }

        // Update balance
        let new_balance = current_balance.pto_balance_hours + hours_to_accrue;
        let accrual_date = now.date();

        sqlx::query!(
            "UPDATE users SET pto_balance_hours = ?, last_accrual_date = ?, updated_at = ? WHERE id = ?",
            new_balance,
            accrual_date,
            now,
            user_id
        )
        .execute(&self.pool)
        .await?;

        // Create history record
        let balance_type_str = PtoBalanceType::Pto.to_string();
        let change_type_str = PtoChangeType::Accrual.to_string();

        let history_row = sqlx::query!(
            r#"
            INSERT INTO pto_balance_history (
                user_id, balance_type, change_type, hours_changed, 
                previous_balance, new_balance, description, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, user_id, balance_type, change_type, hours_changed,
                previous_balance, new_balance, description, related_time_off_id, created_at
            "#,
            user_id,
            balance_type_str,
            change_type_str,
            hours_to_accrue,
            current_balance.pto_balance_hours,
            new_balance,
            "Monthly PTO accrual",
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(PtoBalanceHistory {
            id: history_row.id,
            user_id: history_row.user_id,
            balance_type: balance_type_str
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?,
            change_type: change_type_str
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?,
            hours_changed: history_row.hours_changed as i32,
            previous_balance: history_row.previous_balance as i32,
            new_balance: history_row.new_balance as i32,
            description: history_row.description,
            related_time_off_id: history_row.related_time_off_id,
            created_at: history_row.created_at.unwrap_or(now),
        }))
    }

    /// Use PTO balance for a time-off request
    pub async fn use_balance_for_time_off(
        &self,
        user_id: &str,
        time_off_id: i64,
        balance_type: PtoBalanceType,
        hours_used: i32,
    ) -> Result<PtoBalanceHistory> {
        let now = Utc::now().naive_utc();

        // Get current balance
        let current_balance = self.get_balance(user_id).await?;
        if current_balance.is_none() {
            return Err(anyhow::anyhow!("User not found"));
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

        // Update balance in users table
        let query = format!(
            "UPDATE users SET {} = ?, updated_at = ? WHERE id = ?",
            field_name
        );
        sqlx::query(&query)
            .bind(new_balance)
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Create history record
        let balance_type_str = balance_type.to_string();
        let change_type_str = PtoChangeType::Usage.to_string();
        let hours_changed = -hours_used;

        let history_row = sqlx::query!(
            r#"
            INSERT INTO pto_balance_history (
                user_id, balance_type, change_type, hours_changed, 
                previous_balance, new_balance, description, related_time_off_id, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, user_id, balance_type, change_type, hours_changed,
                previous_balance, new_balance, description, related_time_off_id, created_at
            "#,
            user_id,
            balance_type_str,
            change_type_str,
            hours_changed,
            previous_balance,
            new_balance,
            "Time-off request usage",
            time_off_id,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PtoBalanceHistory {
            id: history_row.id,
            user_id: history_row.user_id,
            balance_type: balance_type_str
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?,
            change_type: change_type_str
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?,
            hours_changed: history_row.hours_changed as i32,
            previous_balance: history_row.previous_balance as i32,
            new_balance: history_row.new_balance as i32,
            description: history_row.description,
            related_time_off_id: history_row.related_time_off_id,
            created_at: history_row.created_at.unwrap_or(now),
        })
    }
}
