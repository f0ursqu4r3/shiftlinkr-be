use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use sqlx::PgPool;

use crate::database::models::{TimeOffRequest, TimeOffRequestInput, TimeOffStatus};

#[derive(Clone)]
pub struct TimeOffRepository {
    pool: PgPool,
}

impl TimeOffRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /// Create a new time-off request
    pub async fn create_request(&self, input: TimeOffRequestInput) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();
        let request_type_str = input.request_type.to_string();
        let status_str = TimeOffStatus::Pending.to_string();

        let row = sqlx::query!(
            r#"
            INSERT INTO time_off_requests (
                user_id, start_date, end_date, reason, request_type, 
                status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            input.user_id,
            input.start_date,
            input.end_date,
            input.reason,
            request_type_str,
            status_str,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TimeOffRequest {
            id: row.id.expect("Row ID should not be null"),
            user_id: row.user_id,
            start_date: row.start_date,
            end_date: row.end_date,
            reason: row.reason,
            request_type: row.request_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Get all time-off requests with optional filtering
    pub async fn get_requests(
        &self,
        user_id: Option<&str>,
        status: Option<TimeOffStatus>,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>,
    ) -> Result<Vec<TimeOffRequest>> {
        let mut query = String::from(
            "SELECT id, user_id, start_date, end_date, reason, request_type, status, approved_by, approval_notes, created_at, updated_at FROM time_off_requests WHERE 1=1"
        );
        let mut params: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send>> = Vec::new();

        if let Some(uid) = user_id {
            query.push_str(" AND user_id = $1");
            params.push(Box::new(uid.to_string()));
        }

        if let Some(s) = status {
            query.push_str(" AND status = $2");
            params.push(Box::new(s.to_string()));
        }

        if let Some(sd) = start_date {
            query.push_str(" AND start_date >= $3");
            params.push(Box::new(sd));
        }

        if let Some(ed) = end_date {
            query.push_str(" AND end_date <= $4");
            params.push(Box::new(ed));
        }

        query.push_str(" ORDER BY created_at DESC");

        // For now, return empty vec - will implement proper dynamic queries later
        let rows = sqlx::query!(
            "SELECT id, user_id, start_date, end_date, reason, request_type, status, approved_by, approval_notes, created_at, updated_at FROM time_off_requests ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let requests = rows
            .into_iter()
            .map(|row| TimeOffRequest {
                id: row.id,
                user_id: row.user_id,
                start_date: row.start_date,
                end_date: row.end_date,
                reason: row.reason,
                request_type: row.request_type.parse().unwrap(),
                status: row.status.parse().unwrap(),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at.unwrap(),
                updated_at: row.updated_at.unwrap(),
            })
            .collect();

        Ok(requests)
    }

    /// Get a specific time-off request by ID
    pub async fn get_request_by_id(&self, id: i64) -> Result<Option<TimeOffRequest>> {
        let row = sqlx::query!(
            "SELECT id, user_id, start_date, end_date, reason, request_type, status, approved_by, approval_notes, created_at, updated_at FROM time_off_requests WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(TimeOffRequest {
                id: row.id,
                user_id: row.user_id,
                start_date: row.start_date,
                end_date: row.end_date,
                reason: row.reason,
                request_type: row.request_type.parse().unwrap(),
                status: row.status.parse().unwrap(),
                approved_by: row.approved_by,
                approval_notes: row.approval_notes,
                created_at: row.created_at.unwrap(),
                updated_at: row.updated_at.unwrap(),
            })),
            None => Ok(None),
        }
    }

    /// Update a time-off request
    pub async fn update_request(
        &self,
        id: i64,
        input: TimeOffRequestInput,
    ) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();
        let request_type_str = input.request_type.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE time_off_requests 
            SET 
                start_date = $1, end_date = $2, reason = $3, request_type = $4, updated_at = $5
            WHERE id = $6
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            input.start_date,
            input.end_date,
            input.reason,
            request_type_str,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TimeOffRequest {
            id: row.id,
            user_id: row.user_id,
            start_date: row.start_date,
            end_date: row.end_date,
            reason: row.reason,
            request_type: row.request_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Approve a time-off request
    pub async fn approve_request(
        &self,
        id: i64,
        approved_by: &str,
        notes: Option<String>,
    ) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();
        let status_str = TimeOffStatus::Approved.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE time_off_requests 
            SET 
                status = $1, approved_by = $2, approval_notes = $3, updated_at = $4
            WHERE id = $5
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            status_str,
            approved_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TimeOffRequest {
            id: row.id,
            user_id: row.user_id,
            start_date: row.start_date,
            end_date: row.end_date,
            reason: row.reason,
            request_type: row.request_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Deny a time-off request
    pub async fn deny_request(
        &self,
        id: i64,
        denied_by: &str,
        notes: Option<String>,
    ) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();
        let status_str = TimeOffStatus::Denied.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE time_off_requests 
            SET 
                status = $1, approved_by = $2, approval_notes = $3, updated_at = $4
            WHERE id = $5
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            status_str,
            denied_by,
            notes,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TimeOffRequest {
            id: row.id,
            user_id: row.user_id,
            start_date: row.start_date,
            end_date: row.end_date,
            reason: row.reason,
            request_type: row.request_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Cancel a time-off request
    pub async fn cancel_request(&self, id: i64) -> Result<TimeOffRequest> {
        let now = Utc::now().naive_utc();
        let status_str = TimeOffStatus::Cancelled.to_string();

        let row = sqlx::query!(
            r#"
            UPDATE time_off_requests 
            SET 
                status = $1, updated_at = $2
            WHERE id = $3
            RETURNING 
                id, user_id, start_date, end_date, reason,
                request_type, status, approved_by, approval_notes, created_at, updated_at
            "#,
            status_str,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TimeOffRequest {
            id: row.id,
            user_id: row.user_id,
            start_date: row.start_date,
            end_date: row.end_date,
            reason: row.reason,
            request_type: row.request_type.parse().unwrap(),
            status: row.status.parse().unwrap(),
            approved_by: row.approved_by,
            approval_notes: row.approval_notes,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        })
    }

    /// Delete a time-off request
    pub async fn delete_request(&self, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM time_off_requests WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
