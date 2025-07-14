use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;

use crate::database::{
    models::{DashboardStats, ShiftStats, TimeOffStats},
    Result,
};

pub struct StatsRepository {
    pool: SqlitePool,
}

impl StatsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // Simple stats that just return basic counts
    pub async fn get_dashboard_stats(
        &self,
        _user_id: Option<String>,
        _start_date: Option<NaiveDateTime>,
        _end_date: Option<NaiveDateTime>,
    ) -> Result<DashboardStats> {
        // For now, return basic counts
        let total_shifts = sqlx::query_scalar!("SELECT COUNT(*) FROM shifts")
            .fetch_one(&self.pool)
            .await? as i64;

        let now = Utc::now().naive_utc();
        let upcoming_shifts =
            sqlx::query_scalar!("SELECT COUNT(*) FROM shifts WHERE start_time > ?", now)
                .fetch_one(&self.pool)
                .await? as i64;

        let pending_time_off_requests =
            sqlx::query_scalar!("SELECT COUNT(*) FROM time_off_requests WHERE status = 'pending'")
                .fetch_one(&self.pool)
                .await? as i64;

        let pending_swap_requests =
            sqlx::query_scalar!("SELECT COUNT(*) FROM shift_swaps WHERE status = 'pending'")
                .fetch_one(&self.pool)
                .await? as i64;

        let approved_time_off =
            sqlx::query_scalar!("SELECT COUNT(*) FROM time_off_requests WHERE status = 'approved'")
                .fetch_one(&self.pool)
                .await? as i64;

        Ok(DashboardStats {
            total_shifts,
            upcoming_shifts,
            pending_time_off_requests,
            pending_swap_requests,
            approved_time_off,
            total_hours: 0.0,     // TODO: Calculate hours
            team_coverage: 100.0, // TODO: Calculate coverage
        })
    }

    // Get shift-specific statistics
    pub async fn get_shift_stats(
        &self,
        _user_id: Option<String>,
        _start_date: Option<NaiveDateTime>,
        _end_date: Option<NaiveDateTime>,
    ) -> Result<ShiftStats> {
        let total_shifts = sqlx::query_scalar!("SELECT COUNT(*) FROM shifts")
            .fetch_one(&self.pool)
            .await? as i64;

        let assigned_shifts =
            sqlx::query_scalar!("SELECT COUNT(DISTINCT shift_id) FROM shift_assignments WHERE status IN ('scheduled', 'confirmed', 'completed')")
                .fetch_one(&self.pool)
                .await? as i64;

        let unassigned_shifts =
            sqlx::query_scalar!("SELECT COUNT(*) FROM shifts WHERE id NOT IN (SELECT DISTINCT shift_id FROM shift_assignments WHERE status IN ('scheduled', 'confirmed', 'completed'))")
                .fetch_one(&self.pool)
                .await? as i64;

        let completed_shifts =
            sqlx::query_scalar!("SELECT COUNT(*) FROM shifts WHERE status = 'completed'")
                .fetch_one(&self.pool)
                .await? as i64;

        let cancelled_shifts =
            sqlx::query_scalar!("SELECT COUNT(*) FROM shifts WHERE status = 'cancelled'")
                .fetch_one(&self.pool)
                .await? as i64;

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
        &self,
        _user_id: Option<String>,
        _start_date: Option<NaiveDateTime>,
        _end_date: Option<NaiveDateTime>,
    ) -> Result<TimeOffStats> {
        let total_requests = sqlx::query_scalar!("SELECT COUNT(*) FROM time_off_requests")
            .fetch_one(&self.pool)
            .await? as i64;

        let approved_requests =
            sqlx::query_scalar!("SELECT COUNT(*) FROM time_off_requests WHERE status = 'approved'")
                .fetch_one(&self.pool)
                .await? as i64;

        let denied_requests =
            sqlx::query_scalar!("SELECT COUNT(*) FROM time_off_requests WHERE status = 'denied'")
                .fetch_one(&self.pool)
                .await? as i64;

        let pending_requests =
            sqlx::query_scalar!("SELECT COUNT(*) FROM time_off_requests WHERE status = 'pending'")
                .fetch_one(&self.pool)
                .await? as i64;

        let cancelled_requests = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM time_off_requests WHERE status = 'cancelled'"
        )
        .fetch_one(&self.pool)
        .await? as i64;

        Ok(TimeOffStats {
            total_requests,
            approved_requests,
            denied_requests,
            pending_requests,
            cancelled_requests,
        })
    }
}
