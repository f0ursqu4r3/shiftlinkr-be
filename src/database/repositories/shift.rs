use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    models::{Shift, ShiftInput, ShiftQuery, ShiftQueryType, ShiftStatus},
    utils::sql,
};

#[derive(Clone)]
pub struct ShiftRepository {
    pool: PgPool,
}

pub struct ShiftFindByFilter {
    pub location_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

impl ShiftRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_shift(&self, input: ShiftInput) -> Result<Shift> {
        let now = Utc::now();
        let row = sqlx::query_as::<_, Shift>(&sql(r#"
            INSERT INTO
                shifts (
                    company_id,
                    title,
                    description,
                    location_id,
                    team_id,
                    start_time,
                    end_time,
                    min_duration_minutes,
                    max_duration_minutes,
                    max_people,
                    status,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id,
                company_id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
        "#))
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

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Shift>> {
        let row = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT
                id,
                company_id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
            FROM
                shifts
            WHERE
                id = ?
        "#))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn find_by_company_id(&self, company_id: Uuid) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT
                s.id,
                s.company_id,
                s.title,
                s.description,
                s.location_id,
                s.team_id,
                s.start_time,
                s.end_time,
                s.min_duration_minutes,
                s.max_duration_minutes,
                s.max_people,
                s.status,
                s.created_at,
                s.updated_at
            FROM
                shifts s
                JOIN locations l ON s.location_id = l.id
            WHERE
                l.company_id = ?
            ORDER BY
                s.start_time
        "#))
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn find_by_location_id(&self, location_id: Uuid) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT
                id,
                company_id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
            FROM
                shifts
            WHERE
                location_id = ?
            ORDER BY
                start_time
        "#))
        .bind(location_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn find_by_team_id(&self, team_id: Uuid) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT
                id,
                company_id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
            FROM
                shifts
            WHERE
                team_id = ?
            ORDER BY
                start_time
        "#))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn find_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        location_id: Option<Uuid>,
    ) -> Result<Vec<Shift>> {
        let rows = if let Some(location_id) = location_id {
            sqlx::query_as::<_, Shift>(&sql(r#"
                SELECT
                    id,
                    company_id,
                    title,
                    description,
                    location_id,
                    team_id,
                    start_time,
                    end_time,
                    min_duration_minutes,
                    max_duration_minutes,
                    max_people,
                    status,
                    created_at,
                    updated_at
                FROM
                    shifts
                WHERE
                    start_time >= ?
                    AND end_time <= ?
                    AND location_id = ?
                ORDER BY
                    start_time
            "#))
            .bind(start_date)
            .bind(end_date)
            .bind(location_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Shift>(&sql(r#"
                SELECT
                    id,
                    company_id,
                    title,
                    description,
                    location_id,
                    team_id,
                    start_time,
                    end_time,
                    min_duration_minutes,
                    max_duration_minutes,
                    max_people,
                    status,
                    created_at,
                    updated_at
                FROM
                    shifts
                WHERE
                    start_time >= ?
                    AND end_time <= ?
                ORDER BY
                    start_time
            "#))
            .bind(start_date)
            .bind(end_date)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn find_open_shifts_by_location(&self, location_id: Uuid) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT
                id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
            FROM
                shifts
            WHERE
                location_id = ?
                AND status = ?
            ORDER BY
                start_time
        "#))
        .bind(location_id)
        .bind(ShiftStatus::Open.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn find_open_shifts(&self) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT
                id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
            FROM
                shifts
            WHERE
                status = ?
            ORDER BY
                start_time
        "#))
        .bind(ShiftStatus::Open.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    pub async fn update_shift(&self, id: Uuid, input: ShiftInput) -> Result<Option<Shift>> {
        let now = Utc::now();
        let row = sqlx::query_as::<_, Shift>(&sql(r#"
            UPDATE
                shifts
            SET
                title = ?,
                company_id = ?
                description = ?,
                location_id = ?,
                team_id = ?,
                start_time = ?,
                end_time = ?,
                min_duration_minutes = ?,
                max_duration_minutes = ?,
                max_people = ?,
                status = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                company_id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
        "#))
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

    pub async fn update_shift_status(
        &self,
        id: Uuid,
        status: ShiftStatus,
    ) -> Result<Option<Shift>> {
        let now = Utc::now();
        let row = sqlx::query_as::<_, Shift>(&sql(r#"
            UPDATE
                shifts
            SET
                status = ?,
                updated_at = ?
            WHERE
                id = ?
            RETURNING
                id,
                company_id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
        "#))
        .bind(status.to_string())
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn delete_shift(&self, id: Uuid) -> Result<Option<()>> {
        let result = sqlx::query(&sql("DELETE FROM shifts WHERE id = ?"))
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(if result.rows_affected() > 0 {
            Some(())
        } else {
            None
        })
    }

    // Get shifts assigned to a specific user through the assignment system
    pub async fn find_shifts_by_user(&self, user_id: Uuid) -> Result<Vec<Shift>> {
        let rows = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT DISTINCT
                s.id,
                s.company_id,
                s.title,
                s.description,
                s.location_id,
                s.team_id,
                s.start_time,
                s.end_time,
                s.min_duration_minutes,
                s.max_duration_minutes,
                s.max_people,
                s.status,
                s.created_at,
                s.updated_at
            FROM
                shifts s
                INNER JOIN shift_proposal_assignments spa ON s.id = spa.shift_id
            WHERE
                spa.user_id = ?
                AND spa.assignment_status = 'accepted'
            ORDER BY
                s.start_time
        "#))
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    // Assign shift using the assignment system (creates a shift assignment)
    pub async fn assign_shift(&self, shift_id: Uuid, _user_id: Uuid) -> Result<Option<Shift>> {
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
    pub async fn unassign_shift(&self, shift_id: Uuid) -> Result<Option<Shift>> {
        match self
            .update_shift_status(shift_id, crate::database::models::ShiftStatus::Open)
            .await
        {
            Ok(shift) => Ok(shift),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_query(&self, filter_query: ShiftQuery) -> Result<Vec<Shift>> {
        let mut query = r#"
            SELECT
                id,
                company_id,
                title,
                description,
                location_id,
                team_id,
                start_time,
                end_time,
                min_duration_minutes,
                max_duration_minutes,
                max_people,
                status,
                created_at,
                updated_at
            FROM
                shifts
            WHERE
        "#
        .to_string();
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        match filter_query.query_type {
            ShiftQueryType::User(user_id) => {
                conditions.push(
                    "id IN (SELECT shift_id FROM shift_proposal_assignments WHERE user_id = ?)",
                );
                params.push(user_id.to_string());
            }
            ShiftQueryType::Location(location_id) => {
                conditions.push("location_id = ?");
                params.push(location_id.to_string());
            }
            ShiftQueryType::Team(team_id) => {
                conditions.push("team_id = ?");
                params.push(team_id.to_string());
            }
            ShiftQueryType::Company(company_id) => {
                conditions.push("company_id = ?");
                params.push(company_id.to_string());
            }
        };

        if let Some(start_date) = filter_query.start_date {
            conditions.push("start_time >= ?");
            params.push(start_date.to_string());
        }
        if let Some(end_date) = filter_query.end_date {
            conditions.push("end_time <= ?");
            params.push(end_date.to_string());
        }
        if let Some(status) = filter_query.status {
            conditions.push("status = ?");
            params.push(status.to_string());
        }

        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));

        query.push_str(" ORDER BY created_at DESC");

        let converted = sql(&query);
        let mut prepared = sqlx::query_as::<_, Shift>(&converted);

        for param in params.into_iter() {
            prepared = prepared.bind(param);
        }
        let shifts = prepared.fetch_all(&self.pool).await?;

        Ok(shifts)
    }
}
