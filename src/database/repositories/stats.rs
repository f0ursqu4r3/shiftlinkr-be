use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::{
    get_pool,
    models::{DashboardStats, ShiftStats, TimeOffStats},
};

// Simple stats that just return basic counts
pub async fn get_dashboard_stats_for_user(
    _user_id: Option<Uuid>,
    _start_date: Option<DateTime<Utc>>,
    _end_date: Option<DateTime<Utc>>,
) -> Result<DashboardStats, sqlx::Error> {
    // For now, return basic counts
    let total_shifts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM shifts")
        .fetch_one(&get_pool().await)
        .await?;

    let now = Utc::now();
    let upcoming_shifts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shifts WHERE start_time > $1")
            .bind(now)
            .fetch_one(&get_pool().await)
            .await?;

    let pending_time_off_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM time_off_requests WHERE status = 'pending'")
            .fetch_one(&get_pool().await)
            .await?;

    let pending_swap_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shift_swaps WHERE status = 'pending'")
            .fetch_one(&get_pool().await)
            .await?;

    let approved_time_off: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM time_off_requests WHERE status = 'approved'")
            .fetch_one(&get_pool().await)
            .await?;

    // Calculate total hours from shifts (convert seconds to hours)
    let total_hours: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(EXTRACT(EPOCH FROM (end_time - start_time)) / 3600), 0)::DOUBLE PRECISION FROM shifts"
        )
        .fetch_one(&get_pool().await)
        .await?;

    // Calculate team coverage as percentage of shifts that are assigned (not open)
    let assigned_shifts_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shifts WHERE status != 'open'")
            .fetch_one(&get_pool().await)
            .await?;

    let team_coverage = if total_shifts > 0 {
        (assigned_shifts_count as f64 / total_shifts as f64) * 100.0
    } else {
        0.0
    };

    Ok(DashboardStats {
        total_shifts,
        upcoming_shifts,
        pending_time_off_requests,
        pending_swap_requests,
        approved_time_off,
        total_hours,
        team_coverage,
    })
}

pub async fn get_dashboard_stats_for_company(
    _company_id: Uuid,
    _start_date: Option<DateTime<Utc>>,
    _end_date: Option<DateTime<Utc>>,
) -> Result<DashboardStats, sqlx::Error> {
    // Implement the logic to fetch dashboard stats for a specific company
    todo!()
}

// Get shift-specific statistics
pub async fn get_shift_stats(
    _user_id: Option<Uuid>,
    _start_date: Option<DateTime<Utc>>,
    _end_date: Option<DateTime<Utc>>,
) -> Result<ShiftStats, sqlx::Error> {
    let total_shifts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM shifts")
        .fetch_one(&get_pool().await)
        .await?;

    // Count shifts that are assigned (not open status)
    let assigned_shifts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM shifts WHERE status IN ('assigned', 'confirmed', 'completed')",
    )
    .fetch_one(&get_pool().await)
    .await?;

    let unassigned_shifts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shifts WHERE status = 'open'")
            .fetch_one(&get_pool().await)
            .await?;

    let completed_shifts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shifts WHERE status = 'completed'")
            .fetch_one(&get_pool().await)
            .await?;

    let cancelled_shifts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shifts WHERE status = 'cancelled'")
            .fetch_one(&get_pool().await)
            .await?;

    Ok(ShiftStats {
        total_shifts,
        assigned_shifts,
        unassigned_shifts,
        completed_shifts,
        cancelled_shifts,
    })
}

// Get time-off statistics
pub async fn get_time_off_stats(
    _user_id: Option<Uuid>,
    _start_date: Option<DateTime<Utc>>,
    _end_date: Option<DateTime<Utc>>,
) -> Result<TimeOffStats, sqlx::Error> {
    let total_requests: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM time_off_requests")
        .fetch_one(&get_pool().await)
        .await?;

    let approved_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM time_off_requests WHERE status = 'approved'")
            .fetch_one(&get_pool().await)
            .await?;

    let denied_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM time_off_requests WHERE status = 'denied'")
            .fetch_one(&get_pool().await)
            .await?;

    let pending_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM time_off_requests WHERE status = 'pending'")
            .fetch_one(&get_pool().await)
            .await?;

    let cancelled_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM time_off_requests WHERE status = 'cancelled'")
            .fetch_one(&get_pool().await)
            .await?;

    Ok(TimeOffStats {
        total_requests,
        approved_requests,
        denied_requests,
        pending_requests,
        cancelled_requests,
    })
}
