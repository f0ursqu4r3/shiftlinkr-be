use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;

use crate::database::models::{TimeOffRequest, TimeOffRequestInput, TimeOffStatus, TimeOffType};

#[derive(Clone)]
pub struct TimeOffRepository {
    pool: SqlitePool,
}

impl TimeOffRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new time-off request
    pub async fn create_request(&self, input: TimeOffRequestInput) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();
        
        let request = sqlx::query_as!(
            TimeOffRequest,
            r#"
            INSERT INTO time_off_requests (
                user_id, start_date, end_date, reason, request_type, 
                status, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type as "request_type: TimeOffType",
                status as "status: TimeOffStatus",
                approved_by, approval_notes, created_at, updated_at
            "#,
            input.user_id,
            input.start_date,
            input.end_date,
            input.reason,
            input.request_type.to_string(),
            TimeOffStatus::Pending.to_string(),
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    /// Get all time-off requests with optional filtering
    pub async fn get_requests(&self, user_id: Option<&str>, status: Option<TimeOffStatus>) -> Result<Vec<TimeOffRequest>> {
        let mut query = "
            SELECT 
                id, user_id, start_date, end_date, reason,
                request_type, status, approved_by, approval_notes,
                created_at, updated_at
            FROM time_off_requests
            WHERE 1=1
        ".to_string();

        let mut params: Vec<String> = Vec::new();

        if let Some(uid) = user_id {
            query.push_str(" AND user_id = ?");
            params.push(uid.to_string());
        }

        if let Some(s) = status {
            query.push_str(" AND status = ?");
            params.push(s.to_string());
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut sql_query = sqlx::query_as::<_, TimeOffRequest>(&query);
        
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let requests = sql_query.fetch_all(&self.pool).await?;
        Ok(requests)
    }

    /// Get a specific time-off request by ID
    pub async fn get_request_by_id(&self, id: i64) -> Result<Option<TimeOffRequest>> {
        let request = sqlx::query_as!(
            TimeOffRequest,
            r#"
            SELECT 
                id, user_id, start_date, end_date, reason,
                request_type as "request_type: TimeOffType",
                status as "status: TimeOffStatus",
                approved_by, approval_notes, created_at, updated_at
            FROM time_off_requests 
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(request)
    }

    /// Update a time-off request
    pub async fn update_request(&self, id: i64, input: TimeOffRequestInput) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();

        let request = sqlx::query_as!(
            TimeOffRequest,
            r#"
            UPDATE time_off_requests 
            SET 
                start_date = ?, end_date = ?, reason = ?, 
                request_type = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type as "request_type: TimeOffType",
                status as "status: TimeOffStatus",
                approved_by, approval_notes, created_at, updated_at
            "#,
            input.start_date,
            input.end_date,
            input.reason,
            input.request_type.to_string(),
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    /// Delete a time-off request
    pub async fn delete_request(&self, id: i64) -> Result<()> {
        sqlx::query!(
            "DELETE FROM time_off_requests WHERE id = ?",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Approve a time-off request
    pub async fn approve_request(&self, id: i64, approved_by: &str, notes: Option<String>) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();

        let request = sqlx::query_as!(
            TimeOffRequest,
            r#"
            UPDATE time_off_requests 
            SET 
                status = ?, approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type as "request_type: TimeOffType",
                status as "status: TimeOffStatus",
                approved_by, approval_notes, created_at, updated_at
            "#,
            TimeOffStatus::Approved.to_string(),
            approved_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    /// Deny a time-off request
    pub async fn deny_request(&self, id: i64, denied_by: &str, notes: String) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();

        let request = sqlx::query_as!(
            TimeOffRequest,
            r#"
            UPDATE time_off_requests 
            SET 
                status = ?, approved_by = ?, approval_notes = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type as "request_type: TimeOffType",
                status as "status: TimeOffStatus",
                approved_by, approval_notes, created_at, updated_at
            "#,
            TimeOffStatus::Denied.to_string(),
            denied_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    /// Check if user has overlapping time-off requests
    pub async fn has_overlapping_requests(&self, user_id: &str, start_date: NaiveDateTime, end_date: NaiveDateTime, exclude_id: Option<i64>) -> Result<bool> {
        let mut query = "
            SELECT COUNT(*) as count
            FROM time_off_requests 
            WHERE user_id = ? 
                AND status IN ('pending', 'approved')
                AND (
                    (start_date <= ? AND end_date >= ?) OR
                    (start_date <= ? AND end_date >= ?) OR
                    (start_date >= ? AND end_date <= ?)
                )
        ".to_string();

        let mut params = vec![
            user_id.to_string(),
            start_date.to_string(),
            start_date.to_string(),
            end_date.to_string(),
            end_date.to_string(),
            start_date.to_string(),
            end_date.to_string(),
        ];

        if let Some(id) = exclude_id {
            query.push_str(" AND id != ?");
            params.push(id.to_string());
        }

        let mut sql_query = sqlx::query_scalar::<_, i64>(&query);
        
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let count = sql_query.fetch_one(&self.pool).await?;
        Ok(count > 0)
    }

    /// Get time-off statistics
    pub async fn get_time_off_stats(&self, user_id: Option<&str>) -> Result<(i64, i64, i64, i64, i64)> {
        let mut query = "
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END) as approved,
                SUM(CASE WHEN status = 'denied' THEN 1 ELSE 0 END) as denied,
                SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) as cancelled
            FROM time_off_requests
        ".to_string();

        let stats = if let Some(uid) = user_id {
            query.push_str(" WHERE user_id = ?");
            sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(&query)
                .bind(uid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(&query)
                .fetch_one(&self.pool)
                .await?
        };

        Ok(stats)
    }

    /// Get time-off requests for a specific date range
    pub async fn get_requests_for_date_range(
        &self,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        user_id: Option<&str>,
    ) -> Result<Vec<TimeOffRequest>> {
        let mut query = "
            SELECT 
                id, user_id, start_date, end_date, reason,
                request_type, status, approved_by, approval_notes,
                created_at, updated_at
            FROM time_off_requests 
            WHERE (start_date <= ? AND end_date >= ?)
        ".to_string();

        let mut params = vec![end_date.to_string(), start_date.to_string()];

        if let Some(uid) = user_id {
            query.push_str(" AND user_id = ?");
            params.push(uid.to_string());
        }

        query.push_str(" ORDER BY start_date ASC");

        let mut sql_query = sqlx::query_as::<_, TimeOffRequest>(&query);
        
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let requests = sql_query.fetch_all(&self.pool).await?;
        Ok(requests)
    }
}
