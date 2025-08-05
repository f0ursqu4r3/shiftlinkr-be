use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::{
    models::{
        AssignmentResponse, AssignmentStatus, Shift, ShiftAssignment, ShiftAssignmentInput,
        UserShiftSchedule, UserShiftScheduleInput,
    },
    pool,
    utils::sql,
};

// User Shift Schedules
pub async fn create_user_schedule(input: UserShiftScheduleInput) -> Result<UserShiftSchedule> {
    let now = Utc::now().naive_utc();
    let schedule = sqlx::query_as::<_, UserShiftSchedule>(
            r#"
            INSERT INTO
                user_shift_schedules (
                    user_id,
                    monday_start,
                    monday_end,
                    tuesday_start,
                    tuesday_end,
                    wednesday_start,
                    wednesday_end,
                    thursday_start,
                    thursday_end,
                    friday_start,
                    friday_end,
                    saturday_start,
                    saturday_end,
                    sunday_start,
                    sunday_end,
                    max_hours_per_week,
                    min_hours_per_week,
                    is_available_for_overtime,
                    created_at,
                    updated_at
                )
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING
                id,
                user_id,
                monday_start,
                monday_end,
                tuesday_start,
                tuesday_end,
                wednesday_start,
                wednesday_end,
                thursday_start,
                thursday_end,
                friday_start,
                friday_end,
                saturday_start,
                saturday_end,
                sunday_start,
                sunday_end,
                max_hours_per_week,
                min_hours_per_week,
                is_available_for_overtime,
                created_at,
                updated_at
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
        .fetch_one(pool())
        .await?;

    Ok(schedule)
}

pub async fn get_user_schedule(user_id: Uuid) -> Result<Option<UserShiftSchedule>> {
    let schedule = sqlx::query_as::<_, UserShiftSchedule>(
        r#"
            SELECT
                id,
                user_id,
                monday_start,
                monday_end,
                tuesday_start,
                tuesday_end,
                wednesday_start,
                wednesday_end,
                thursday_start,
                thursday_end,
                friday_start,
                friday_end,
                saturday_start,
                saturday_end,
                sunday_start,
                sunday_end,
                max_hours_per_week,
                min_hours_per_week,
                is_available_for_overtime,
                created_at,
                updated_at
            FROM
                user_shift_schedules
            WHERE
                user_id = $1
            LIMIT 1
            "#,
    )
    .bind(user_id)
    .fetch_optional(pool())
    .await?;

    Ok(schedule)
}

pub async fn update_user_schedule(
    user_id: Uuid,
    input: UserShiftScheduleInput,
) -> Result<Option<UserShiftSchedule>> {
    let now = Utc::now().naive_utc();
    let schedule = sqlx::query_as::<_, UserShiftSchedule>(
        r#"
            UPDATE
                user_shift_schedules
            SET
                monday_start = $1,
                monday_end = $2,
                tuesday_start = $3,
                tuesday_end = $4,
                wednesday_start = $5,
                wednesday_end = $6,
                thursday_start = $7,
                thursday_end = $8,
                friday_start = $9,
                friday_end = $10,
                saturday_start = $11,
                saturday_end = $12,
                sunday_start = $13,
                sunday_end = $14,
                max_hours_per_week = $15,
                min_hours_per_week = $16,
                is_available_for_overtime = $17,
                updated_at = $18
            WHERE
                user_id = $19
            RETURNING
                id,
                user_id,
                monday_start,
                monday_end,
                tuesday_start,
                tuesday_end,
                wednesday_start,
                wednesday_end,
                thursday_start,
                thursday_end,
                friday_start,
                friday_end,
                saturday_start,
                saturday_end,
                sunday_start,
                sunday_end,
                max_hours_per_week,
                min_hours_per_week,
                is_available_for_overtime,
                created_at,
                updated_at
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
    .fetch_optional(pool())
    .await?;

    Ok(schedule)
}

pub async fn delete_user_schedule(user_id: Uuid) -> Result<Option<()>> {
    let result = sqlx::query("DELETE FROM user_shift_schedules WHERE user_id = $1")
        .bind(user_id)
        .execute(pool())
        .await?;

    if result.rows_affected() > 0 {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

// Shift Assignments
pub async fn create_shift_assignment(
    assigned_by_user_id: Uuid,
    input: ShiftAssignmentInput,
) -> Result<ShiftAssignment> {
    let now = Utc::now().naive_utc();
    let assignment = sqlx::query_as::<_, ShiftAssignment>(&sql(r#"
            INSERT INTO
                shift_proposal_assignments (
                    shift_id,
                    user_id,
                    assigned_by,
                    assignment_status,
                    acceptance_deadline,
                    response,
                    response_notes,
                    created_at,
                    updated_at
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
        "#))
    .bind(input.shift_id)
    .bind(&input.user_id)
    .bind(assigned_by_user_id)
    .bind(AssignmentStatus::Pending.to_string())
    .bind(input.acceptance_deadline)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(now)
    .bind(now)
    .fetch_one(pool())
    .await?;

    Ok(assignment)
}

pub async fn get_user_shift_suggestions(user_id: Uuid) -> Result<Vec<Shift>> {
    // This function finds open shifts that match a user's availability and don't conflict with their existing schedule.
    // A more advanced implementation could also factor in user skills, location preferences, and other constraints.
    let suggestions = sqlx::query_as::<_, Shift>(&sql(r#"
            SELECT
                s.id
                s.title
                s.description
                s.location_id
                s.team_id
                s.start_time
                s.end_time
                s.min_duration_minutes
                s.max_duration_minutes
                s.max_people
                s.status
                s.created_at
                s.updated_at
            FROM
                shifts s
                JOIN user_shift_schedules uss ON uss.user_id = ?
            WHERE
                -- Consider only open shifts in the near future (e.g., next 30 days)
                s.status = 'open'
                AND s.start_time BETWEEN NOW() AND NOW() + INTERVAL '30 days'
                
                -- Check if the shift falls within the user's availability for that day of the week.
                -- This handles cases where availability for a day is not set (NULL).
                AND CASE EXTRACT(ISODOW FROM s.start_time)
                    WHEN 1 THEN uss.monday_start IS NOT NULL AND s.start_time::time >= uss.monday_start AND s.end_time::time <= uss.monday_end
                    WHEN 2 THEN uss.tuesday_start IS NOT NULL AND s.start_time::time >= uss.tuesday_start AND s.end_time::time <= uss.tuesday_end
                    WHEN 3 THEN uss.wednesday_start IS NOT NULL AND s.start_time::time >= uss.wednesday_start AND s.end_time::time <= uss.wednesday_end
                    WHEN 4 THEN uss.thursday_start IS NOT NULL AND s.start_time::time >= uss.thursday_start AND s.end_time::time <= uss.thursday_end
                    WHEN 5 THEN uss.friday_start IS NOT NULL AND s.start_time::time >= uss.friday_start AND s.end_time::time <= uss.friday_end
                    WHEN 6 THEN uss.saturday_start IS NOT NULL AND s.start_time::time >= uss.saturday_start AND s.end_time::time <= uss.saturday_end
                    WHEN 7 THEN uss.sunday_start IS NOT NULL AND s.start_time::time >= uss.sunday_start AND s.end_time::time <= uss.sunday_end
                    ELSE FALSE
                END

                -- Ensure the user is not already assigned to an overlapping shift.
                AND NOT EXISTS (
                    SELECT 1
                    FROM shift_assignments sa
                    JOIN shifts assigned_shift ON sa.shift_id = assigned_shift.id
                    WHERE sa.user_id = ?
                      AND sa.assignment_status = 'accepted' -- Consider only accepted assignments for conflicts
                      AND assigned_shift.start_time < s.end_time AND assigned_shift.end_time > s.start_time
                )
            ORDER BY s.start_time
            LIMIT 20
        "#))
        .bind(user_id)
        .fetch_all(pool())
        .await?;

    Ok(suggestions)
}

pub async fn get_shift_assignment(assignment_id: Uuid) -> Result<Option<ShiftAssignment>> {
    let assignment = sqlx::query_as::<_, ShiftAssignment>(
        r#"
            SELECT
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
            FROM
                shift_proposal_assignments
            WHERE
                id = $1
            "#,
    )
    .bind(assignment_id)
    .fetch_optional(pool())
    .await?;

    Ok(assignment)
}

pub async fn get_shift_assignments_by_shift(shift_id: Uuid) -> Result<Vec<ShiftAssignment>> {
    let assignments = sqlx::query_as::<_, ShiftAssignment>(
        r#"
            SELECT
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
            FROM
                shift_proposal_assignments
            WHERE
                shift_id = $1
            ORDER BY
                created_at
            "#,
    )
    .bind(shift_id)
    .fetch_all(pool())
    .await?;

    Ok(assignments)
}

pub async fn get_shift_assignments_by_user(user_id: Uuid) -> Result<Vec<ShiftAssignment>> {
    let assignments = sqlx::query_as::<_, ShiftAssignment>(
        r#"
            SELECT
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
            FROM
                shift_proposal_assignments
            WHERE
                user_id = $1
            ORDER BY
                created_at DESC
            "#,
    )
    .bind(user_id)
    .fetch_all(pool())
    .await?;

    Ok(assignments)
}

pub async fn get_pending_assignments_for_user(user_id: Uuid) -> Result<Vec<ShiftAssignment>> {
    let assignments = sqlx::query_as::<_, ShiftAssignment>(
        r#"
            SELECT
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
            FROM
                shift_proposal_assignments
            WHERE
                user_id = $1 AND assignment_status = 'pending'
            ORDER BY
                acceptance_deadline ASC, created_at
            "#,
    )
    .bind(user_id)
    .fetch_all(pool())
    .await?;

    Ok(assignments)
}

pub async fn respond_to_assignment(
    assignment_id: Uuid,
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
            UPDATE
                shift_proposal_assignments
            SET
                assignment_status = $1,
                response = $2,
                response_notes = $3,
                updated_at = $4
            WHERE
                id = $5
            RETURNING
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
            "#,
    )
    .bind(status.to_string())
    .bind(response.to_string())
    .bind(response_notes)
    .bind(now)
    .bind(assignment_id)
    .fetch_optional(pool())
    .await?;

    Ok(assignment)
}

pub async fn cancel_assignment(assignment_id: Uuid) -> Result<Option<ShiftAssignment>> {
    let now = Utc::now().naive_utc();
    let assignment = sqlx::query_as::<_, ShiftAssignment>(
        r#"
            UPDATE
                shift_proposal_assignments
            SET
                assignment_status = 'cancelled',
                updated_at = $1
            WHERE
                id = $2
            RETURNING
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
            "#,
    )
    .bind(now)
    .bind(assignment_id)
    .fetch_optional(pool())
    .await?;

    Ok(assignment)
}

pub async fn expire_overdue_assignments() -> Result<Vec<ShiftAssignment>> {
    let now = Utc::now().naive_utc();
    let assignments = sqlx::query_as::<_, ShiftAssignment>(
        r#"
            UPDATE
                shift_proposal_assignments
            SET
                assignment_status = 'expired',
                updated_at = $1
            WHERE
                assignment_status = 'pending'
                AND acceptance_deadline IS NOT NULL
                AND acceptance_deadline < $2
            RETURNING
                id,
                shift_id,
                user_id,
                assigned_by,
                assignment_status,
                acceptance_deadline,
                response,
                response_notes,
                created_at,
                updated_at
            "#,
    )
    .bind(now)
    .bind(now)
    .fetch_all(pool())
    .await?;

    Ok(assignments)
}
