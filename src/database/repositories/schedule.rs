use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;

use crate::database::models::{
    AssignmentResponse, AssignmentStatus, ShiftAssignment, ShiftAssignmentInput, UserShiftSchedule,
    UserShiftScheduleInput,
};

pub struct ScheduleRepository {
    pool: PgPool,
}

impl ScheduleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // User Shift Schedules
    pub async fn create_user_schedule(
        &self,
        input: UserShiftScheduleInput,
    ) -> Result<UserShiftSchedule> {
        let now = Utc::now().naive_utc();
        let schedule = sqlx::query_as::<_, UserShiftSchedule>(
            r#"
            INSERT INTO user_shift_schedules (
                user_id, monday_start, monday_end, tuesday_start, tuesday_end,
                wednesday_start, wednesday_end, thursday_start, thursday_end,
                friday_start, friday_end, saturday_start, saturday_end,
                sunday_start, sunday_end, max_hours_per_week, min_hours_per_week,
                is_available_for_overtime, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, user_id, monday_start, monday_end, tuesday_start, tuesday_end,
                      wednesday_start, wednesday_end, thursday_start, thursday_end,
                      friday_start, friday_end, saturday_start, saturday_end,
                      sunday_start, sunday_end, max_hours_per_week, min_hours_per_week,
                      is_available_for_overtime, created_at, updated_at
            "#,
        )
        .bind(&input.user_id)
        .bind(input.monday_start)
        .bind(input.monday_end)
        .bind(input.tuesday_start)
        .bind(input.tuesday_end)
        .bind(input.wednesday_start)
        .bind(input.wednesday_end)
        .bind(input.thursday_start)
        .bind(input.thursday_end)
        .bind(input.friday_start)
        .bind(input.friday_end)
        .bind(input.saturday_start)
        .bind(input.saturday_end)
        .bind(input.sunday_start)
        .bind(input.sunday_end)
        .bind(input.max_hours_per_week)
        .bind(input.min_hours_per_week)
        .bind(input.is_available_for_overtime)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(schedule)
    }

    pub async fn get_user_schedule(&self, user_id: &str) -> Result<Option<UserShiftSchedule>> {
        let schedule = sqlx::query_as::<_, UserShiftSchedule>(
            r#"
            SELECT id, user_id, monday_start, monday_end, tuesday_start, tuesday_end,
                   wednesday_start, wednesday_end, thursday_start, thursday_end,
                   friday_start, friday_end, saturday_start, saturday_end,
                   sunday_start, sunday_end, max_hours_per_week, min_hours_per_week,
                   is_available_for_overtime, created_at, updated_at
            FROM user_shift_schedules WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(schedule)
    }

    pub async fn update_user_schedule(
        &self,
        user_id: &str,
        input: UserShiftScheduleInput,
    ) -> Result<Option<UserShiftSchedule>> {
        let now = Utc::now().naive_utc();
        let schedule = sqlx::query_as::<_, UserShiftSchedule>(
            r#"
            UPDATE user_shift_schedules SET
                monday_start = ?, monday_end = ?, tuesday_start = ?, tuesday_end = ?,
                wednesday_start = ?, wednesday_end = ?, thursday_start = ?, thursday_end = ?,
                friday_start = ?, friday_end = ?, saturday_start = ?, saturday_end = ?,
                sunday_start = ?, sunday_end = ?, max_hours_per_week = ?, min_hours_per_week = ?,
                is_available_for_overtime = ?, updated_at = ?
            WHERE user_id = ?
            RETURNING id, user_id, monday_start, monday_end, tuesday_start, tuesday_end,
                      wednesday_start, wednesday_end, thursday_start, thursday_end,
                      friday_start, friday_end, saturday_start, saturday_end,
                      sunday_start, sunday_end, max_hours_per_week, min_hours_per_week,
                      is_available_for_overtime, created_at, updated_at
            "#,
        )
        .bind(input.monday_start)
        .bind(input.monday_end)
        .bind(input.tuesday_start)
        .bind(input.tuesday_end)
        .bind(input.wednesday_start)
        .bind(input.wednesday_end)
        .bind(input.thursday_start)
        .bind(input.thursday_end)
        .bind(input.friday_start)
        .bind(input.friday_end)
        .bind(input.saturday_start)
        .bind(input.saturday_end)
        .bind(input.sunday_start)
        .bind(input.sunday_end)
        .bind(input.max_hours_per_week)
        .bind(input.min_hours_per_week)
        .bind(input.is_available_for_overtime)
        .bind(now)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(schedule)
    }

    pub async fn delete_user_schedule(&self, user_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM user_shift_schedules WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Shift Assignments
    pub async fn create_shift_assignment(
        &self,
        input: ShiftAssignmentInput,
    ) -> Result<ShiftAssignment> {
        let now = Utc::now().naive_utc();
        let assignment = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            INSERT INTO shift_proposal_assignments (
                shift_id, user_id, assigned_by, assignment_status, 
                acceptance_deadline, response, response_notes, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, shift_id, user_id, assigned_by, assignment_status,
                      acceptance_deadline, response, response_notes, created_at, updated_at
            "#,
        )
        .bind(input.shift_id)
        .bind(&input.user_id)
        .bind(&input.assigned_by)
        .bind(AssignmentStatus::Pending.to_string())
        .bind(input.acceptance_deadline)
        .bind(None::<String>)
        .bind(None::<String>)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(assignment)
    }

    pub async fn get_shift_assignment(&self, id: i64) -> Result<Option<ShiftAssignment>> {
        let assignment = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            SELECT id, shift_id, user_id, assigned_by, assignment_status,
                   acceptance_deadline, response, response_notes, created_at, updated_at
            FROM shift_proposal_assignments WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(assignment)
    }

    pub async fn get_shift_assignments_by_shift(
        &self,
        shift_id: i64,
    ) -> Result<Vec<ShiftAssignment>> {
        let assignments = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            SELECT id, shift_id, user_id, assigned_by, assignment_status,
                   acceptance_deadline, response, response_notes, created_at, updated_at
            FROM shift_proposal_assignments WHERE shift_id = ? ORDER BY created_at
            "#,
        )
        .bind(shift_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(assignments)
    }

    pub async fn get_shift_assignments_by_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<ShiftAssignment>> {
        let assignments = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            SELECT id, shift_id, user_id, assigned_by, assignment_status,
                   acceptance_deadline, response, response_notes, created_at, updated_at
            FROM shift_proposal_assignments WHERE user_id = ? ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(assignments)
    }

    pub async fn get_pending_assignments_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<ShiftAssignment>> {
        let assignments = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            SELECT id, shift_id, user_id, assigned_by, assignment_status,
                   acceptance_deadline, response, response_notes, created_at, updated_at
            FROM shift_proposal_assignments 
            WHERE user_id = ? AND assignment_status = 'pending'
            ORDER BY acceptance_deadline ASC, created_at
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(assignments)
    }

    pub async fn respond_to_assignment(
        &self,
        assignment_id: i64,
        response: AssignmentResponse,
        response_notes: Option<String>,
    ) -> Result<Option<ShiftAssignment>> {
        let now = Utc::now().naive_utc();
        let status = match response {
            AssignmentResponse::Accept => AssignmentStatus::Accepted,
            AssignmentResponse::Decline => AssignmentStatus::Declined,
        };

        let assignment = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            UPDATE shift_proposal_assignments SET
                assignment_status = ?, response = ?, response_notes = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, shift_id, user_id, assigned_by, assignment_status,
                      acceptance_deadline, response, response_notes, created_at, updated_at
            "#,
        )
        .bind(status.to_string())
        .bind(response.to_string())
        .bind(response_notes)
        .bind(now)
        .bind(assignment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(assignment)
    }

    pub async fn cancel_assignment(&self, assignment_id: i64) -> Result<Option<ShiftAssignment>> {
        let now = Utc::now().naive_utc();
        let assignment = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            UPDATE shift_proposal_assignments SET
                assignment_status = 'cancelled', updated_at = ?
            WHERE id = ?
            RETURNING id, shift_id, user_id, assigned_by, assignment_status,
                      acceptance_deadline, response, response_notes, created_at, updated_at
            "#,
        )
        .bind(now)
        .bind(assignment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(assignment)
    }

    pub async fn expire_overdue_assignments(&self) -> Result<Vec<ShiftAssignment>> {
        let now = Utc::now().naive_utc();
        let assignments = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            UPDATE shift_proposal_assignments SET
                assignment_status = 'expired', updated_at = ?
            WHERE assignment_status = 'pending' 
              AND acceptance_deadline IS NOT NULL 
              AND acceptance_deadline < ?
            RETURNING id, shift_id, user_id, assigned_by, assignment_status,
                      acceptance_deadline, response, response_notes, created_at, updated_at
            "#,
        )
        .bind(now)
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(assignments)
    }
}
