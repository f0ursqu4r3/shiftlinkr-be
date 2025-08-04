use anyhow::Result;
use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    models::{TimeOffRequest, TimeOffRequestInput, TimeOffStatus},
    utils::sql,
};

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
        let now = Utc::now();
        let request_type_str = input.request_type.to_string();
        let status_str = TimeOffStatus::Pending.to_string();

        let time_off_request = sqlx::query_as::<_, TimeOffRequest>(&sql(r#"
            INSERT INTO
                time_off_requests (
                    user_id,
                    company_id,
                    start_date,
                    end_date,
                    reason,
                    request_type,
                    status,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id,
                user_id,
                company_id,
                start_date,
                end_date,
                reason,
                request_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
        "#))
        .bind(input.user_id)
        .bind(input.company_id)
        .bind(input.start_date)
        .bind(input.end_date)
        .bind(input.reason)
        .bind(request_type_str)
        .bind(status_str)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(time_off_request)
    }

    /// Get all time-off requests with optional filtering
    pub async fn get_requests(
        &self,
        user_id: Option<Uuid>,
        status: Option<TimeOffStatus>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Vec<TimeOffRequest>> {
        let mut query = r#"
            SELECT
                id,
                user_id,
                start_date,
                end_date,
                reason,
                request_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                time_off_requests
            "#
        .to_string();

        let mut params = Vec::new();
        let mut conditions = vec![];

        if let Some(uid) = user_id {
            conditions.push(format!("user_id = ${}", params.len() + 1));
            params.push(uid.to_string());
        }

        if let Some(s) = status {
            conditions.push(format!("status = ${}", params.len() + 1));
            params.push(s.to_string());
        }

        if let Some(sd) = start_date {
            conditions.push(format!("start_date >= ${}", params.len() + 1));
            params.push(sd.to_string());
        }

        if let Some(ed) = end_date {
            conditions.push(format!("end_date <= ${}", params.len() + 1));
            params.push(ed.to_string());
        }

        if !params.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut prepared = sqlx::query_as::<_, TimeOffRequest>(&query);
        for param in params {
            prepared = prepared.bind(param);
        }

        let requests = prepared.fetch_all(&self.pool).await?;

        Ok(requests)
    }

    /// Get a specific time-off request by ID
    pub async fn get_request_by_id(&self, id: Uuid) -> Result<Option<TimeOffRequest>> {
        let time_off_request = sqlx::query_as::<_, TimeOffRequest>(
            r#"
            SELECT
                id,
                user_id,
                start_date,
                end_date,
                reason,
                request_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            FROM
                time_off_requests
            WHERE
                id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(time_off_request)
    }

    /// Update a time-off request
    pub async fn update_request(
        &self,
        id: Uuid,
        input: TimeOffRequestInput,
    ) -> Result<TimeOffRequest> {
        let now = Utc::now();
        let request_type_str = input.request_type.to_string();

        let time_off_request = sqlx::query_as::<_, TimeOffRequest>(
            r#"
            UPDATE
                time_off_requests
            SET
                start_date = $1,
                end_date = $2,
                reason = $3,
                request_type = $4,
                updated_at = $5
            WHERE
                id = $6
            RETURNING
                id,
                user_id,
                start_date,
                end_date,
                reason,
                request_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(input.start_date)
        .bind(input.end_date)
        .bind(input.reason)
        .bind(request_type_str)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(time_off_request)
    }

    /// Approve a time-off request
    pub async fn approve_request(
        &self,
        id: Uuid,
        approved_by: Uuid,
        notes: Option<String>,
    ) -> Result<TimeOffRequest> {
        let now = Utc::now();
        let status_str = TimeOffStatus::Approved.to_string();

        let time_off_request = sqlx::query_as::<_, TimeOffRequest>(
            r#"
            UPDATE time_off_requests
            SET
                status = $1,
                approved_by = $2,
                approval_notes = $3,
                updated_at = $4
            WHERE
                id = $5
            RETURNING
                id,
                user_id,
                start_date,
                end_date,
                reason,
                request_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(status_str)
        .bind(approved_by)
        .bind(notes)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(time_off_request)
    }

    /// Deny a time-off request
    pub async fn deny_request(
        &self,
        id: Uuid,
        denied_by: Uuid,
        notes: Option<String>,
    ) -> Result<TimeOffRequest> {
        let now = Utc::now();
        let status_str = TimeOffStatus::Denied.to_string();

        let time_off_request = sqlx::query_as::<_, TimeOffRequest>(
            r#"
            UPDATE
                time_off_requests
            SET
                status = $1,
                approved_by = $2,
                approval_notes = $3,
                updated_at = $4
            WHERE
                id = $5
            RETURNING
                id,
                user_id,
                start_date,
                end_date,
                reason,
                request_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(status_str)
        .bind(denied_by)
        .bind(notes)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(time_off_request)
    }

    /// Cancel a time-off request
    pub async fn cancel_request(&self, id: Uuid) -> Result<TimeOffRequest> {
        let now = Utc::now();
        let status_str = TimeOffStatus::Cancelled.to_string();

        let time_off_request = sqlx::query_as::<_, TimeOffRequest>(
            r#"
            UPDATE time_off_requests
            SET
                status = $1,
                updated_at = $2
            WHERE
                id = $3
            RETURNING
                id,
                user_id,
                start_date,
                end_date,
                reason,
                request_type,
                status,
                approved_by,
                approval_notes,
                created_at,
                updated_at
            "#,
        )
        .bind(status_str)
        .bind(now)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(time_off_request)
    }

    /// Delete a time-off request
    pub async fn delete_request(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM time_off_requests WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
