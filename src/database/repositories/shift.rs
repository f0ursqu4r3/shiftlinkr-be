use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use sqlx::PgPool;

use crate::database::models::{Shift, ShiftInput, ShiftStatus};
use crate::database::types::ShiftRow;

pub struct ShiftRepository {
    pool: PgPool,
}

impl ShiftRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_shift(&self, input: ShiftInput) -> Result<Shift> {
        let now = Utc::now().naive_utc();
        let row = sqlx::query_as::<_, ShiftRow>(
            r#"
            INSERT INTO shifts (title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at
            "#
        )
        .bind(input.title)
        .bind(input.description)
        .bind(input.location_id)
        .bind(input.team_id)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(input.min_duration_minutes)
        .bind(input.max_duration_minutes)
        .bind(input.max_people)
        .bind(ShiftStatus::Open.to_string())
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_shift_by_id(&self, id: i64) -> Result<Option<Shift>> {
        let row = sqlx::query_as::<_, ShiftRow>(
            "SELECT id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at FROM shifts WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_shifts_by_location(&self, location_id: i64) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, ShiftRow>(
            "SELECT id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at FROM shifts WHERE location_id = $1 ORDER BY start_time"
        )
        .bind(location_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn get_shifts_by_team(&self, team_id: i64) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, ShiftRow>(
            "SELECT id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at FROM shifts WHERE team_id = $1 ORDER BY start_time"
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn get_shifts_by_date_range(
        &self,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        location_id: Option<i64>,
    ) -> Result<Vec<Shift>> {
        let rows = if let Some(location_id) = location_id {
            sqlx::query_as::<_, ShiftRow>(
                "SELECT id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at FROM shifts WHERE start_time >= $1 AND end_time <= $2 AND location_id = $3 ORDER BY start_time"
            )
            .bind(start_date)
            .bind(end_date)
            .bind(location_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ShiftRow>(
                "SELECT id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at FROM shifts WHERE start_time >= $1 AND end_time <= $2 ORDER BY start_time"
            )
            .bind(start_date)
            .bind(end_date)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn get_open_shifts_by_location(&self, location_id: i64) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, ShiftRow>(
            "SELECT id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at FROM shifts WHERE location_id = $1 AND status = $2 ORDER BY start_time"
        )
        .bind(location_id)
        .bind(ShiftStatus::Open.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn get_open_shifts(&self) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, ShiftRow>(
            "SELECT id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at FROM shifts WHERE status = $1 ORDER BY start_time"
        )
        .bind(ShiftStatus::Open.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn update_shift(&self, id: i64, input: ShiftInput) -> Result<Option<Shift>> {
        let now = Utc::now().naive_utc();
        let row = sqlx::query_as::<_, ShiftRow>(
            r#"
            UPDATE shifts 
            SET title = $1, description = $2, location_id = $3, team_id = $4, start_time = $5, end_time = $6, min_duration_minutes = $7, max_duration_minutes = $8, max_people = $9, status = $10, created_at = $11, updated_at = $12
            WHERE id = $13
            RETURNING id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at
            "#
        )
        .bind(input.title)
        .bind(input.description)
        .bind(input.location_id)
        .bind(input.team_id)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(input.min_duration_minutes)
        .bind(input.max_duration_minutes)
        .bind(input.max_people)
        .bind(input.status.to_string())
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn update_shift_status(&self, id: i64, status: ShiftStatus) -> Result<Option<Shift>> {
        let now = Utc::now().naive_utc();
        let row = sqlx::query_as::<_, ShiftRow>(
            r#"
            UPDATE shifts 
            SET status = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, title, description, location_id, team_id, start_time, end_time, min_duration_minutes, max_duration_minutes, max_people, status, created_at, updated_at
            "#
        )
        .bind(status.to_string())
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn delete_shift(&self, id: i64) -> Result<bool> {
        let result = sqlx::query!("DELETE FROM shifts WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Get shifts assigned to a specific user through the assignment system
    pub async fn get_shifts_by_user(&self, user_id: i64) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, ShiftRow>(
            r#"
            SELECT DISTINCT s.id, s.title, s.description, s.location_id, s.team_id,
                   s.start_time, s.end_time, s.min_duration_minutes, s.max_duration_minutes,
                   s.max_people, s.status, s.created_at, s.updated_at
            FROM shifts s
            INNER JOIN shift_proposal_assignments spa ON s.id = spa.shift_id
            WHERE spa.user_id = $1 AND spa.assignment_status = 'accepted'
            ORDER BY s.start_time
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    // Assign shift using the assignment system (creates a shift assignment)
    pub async fn assign_shift(&self, shift_id: i64, _user_id: i64) -> Result<Option<Shift>> {
        // This is a simplified direct assignment - in practice, you might want to use the schedule repository
        // For now, we'll update the shift status to assigned
        match self
            .update_shift_status(shift_id, crate::database::models::ShiftStatus::Assigned)
            .await
        {
            Ok(shift) => Ok(shift),
            Err(e) => Err(e),
        }
    }

    // Unassign shift by updating status back to open
    pub async fn unassign_shift(&self, shift_id: i64) -> Result<Option<Shift>> {
        match self
            .update_shift_status(shift_id, crate::database::models::ShiftStatus::Open)
            .await
        {
            Ok(shift) => Ok(shift),
            Err(e) => Err(e),
        }
    }
}
