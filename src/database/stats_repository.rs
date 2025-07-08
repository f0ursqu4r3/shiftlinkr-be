use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;

use crate::database::models::{DashboardStats, ShiftStats, TimeOffStats};

#[derive(Clone)]
pub struct StatsRepository {
    pool: SqlitePool,
}

impl StatsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get dashboard statistics for a user (filtered by permissions)
    pub async fn get_dashboard_stats(
        &self, 
        user_id: Option<&str>,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>
    ) -> Result<DashboardStats> {
        let (start_filter, end_filter) = self.get_date_filters(start_date, end_date);
        
        // Total shifts query
        let total_shifts = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id = ? AND start_time >= ? AND end_time <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Upcoming shifts (future shifts)
        let now = Utc::now().naive_utc();
        let upcoming_shifts = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id = ? AND start_time > ? AND start_time >= ? AND end_time <= ?",
                uid,
                now,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE start_time > ? AND start_time >= ? AND end_time <= ?",
                now,
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Pending time-off requests
        let pending_time_off_requests = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE user_id = ? AND status = 'pending' AND start_date >= ? AND end_date <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE status = 'pending' AND start_date >= ? AND end_date <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Pending swap requests
        let pending_swap_requests = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shift_swaps WHERE (requesting_user_id = ? OR target_user_id = ?) AND status IN ('pending', 'open') AND created_at >= ? AND created_at <= ?",
                uid,
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shift_swaps WHERE status IN ('pending', 'open') AND created_at >= ? AND created_at <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Approved time-off requests
        let approved_time_off = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE user_id = ? AND status = 'approved' AND start_date >= ? AND end_date <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE status = 'approved' AND start_date >= ? AND end_date <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Calculate total hours
        let total_hours = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                r#"
                SELECT COALESCE(SUM((julianday(end_time) - julianday(start_time)) * 24), 0) as total_hours
                FROM shifts 
                WHERE assigned_user_id = ? AND start_time >= ? AND end_time <= ?
                "#,
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                r#"
                SELECT COALESCE(SUM((julianday(end_time) - julianday(start_time)) * 24), 0) as total_hours
                FROM shifts 
                WHERE start_time >= ? AND end_time <= ?
                "#,
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0.0);

        // Team coverage calculation (percentage of shifts that are assigned)
        let total_shifts_f = total_shifts as f64;
        let assigned_shifts = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id IS NOT NULL AND assigned_user_id = ? AND start_time >= ? AND end_time <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id IS NOT NULL AND start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as f64;

        let team_coverage = if total_shifts_f > 0.0 {
            (assigned_shifts / total_shifts_f) * 100.0
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

    /// Get shift statistics
    pub async fn get_shift_stats(
        &self,
        user_id: Option<&str>,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>
    ) -> Result<ShiftStats> {
        let (start_filter, end_filter) = self.get_date_filters(start_date, end_date);

        let total_shifts = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id = ? AND start_time >= ? AND end_time <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let assigned_shifts = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id IS NOT NULL AND assigned_user_id = ? AND start_time >= ? AND end_time <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id IS NOT NULL AND start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let unassigned_shifts = if let Some(uid) = user_id {
            // For individual users, unassigned shifts they could potentially take
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id IS NULL AND start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id IS NULL AND start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let completed_shifts = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id = ? AND status = 'completed' AND start_time >= ? AND end_time <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE status = 'completed' AND start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let cancelled_shifts = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE assigned_user_id = ? AND status = 'cancelled' AND start_time >= ? AND end_time <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shifts WHERE status = 'cancelled' AND start_time >= ? AND end_time <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(ShiftStats {
            total_shifts,
            assigned_shifts,
            unassigned_shifts,
            completed_shifts,
            cancelled_shifts,
        })
    }

    /// Get time-off statistics
    pub async fn get_time_off_stats(
        &self,
        user_id: Option<&str>,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>
    ) -> Result<TimeOffStats> {
        let (start_filter, end_filter) = self.get_date_filters(start_date, end_date);

        let total_requests = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE user_id = ? AND start_date >= ? AND end_date <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE start_date >= ? AND end_date <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let approved_requests = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE user_id = ? AND status = 'approved' AND start_date >= ? AND end_date <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE status = 'approved' AND start_date >= ? AND end_date <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let denied_requests = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE user_id = ? AND status = 'denied' AND start_date >= ? AND end_date <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE status = 'denied' AND start_date >= ? AND end_date <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let pending_requests = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE user_id = ? AND status = 'pending' AND start_date >= ? AND end_date <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE status = 'pending' AND start_date >= ? AND end_date <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let cancelled_requests = if let Some(uid) = user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE user_id = ? AND status = 'cancelled' AND start_date >= ? AND end_date <= ?",
                uid,
                start_filter,
                end_filter
            )
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM time_off_requests WHERE status = 'cancelled' AND start_date >= ? AND end_date <= ?",
                start_filter,
                end_filter
            )
        }
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(TimeOffStats {
            total_requests,
            approved_requests,
            denied_requests,
            pending_requests,
            cancelled_requests,
        })
    }

    /// Helper method to get date filters with defaults
    fn get_date_filters(&self, start_date: Option<NaiveDateTime>, end_date: Option<NaiveDateTime>) -> (NaiveDateTime, NaiveDateTime) {
        let now = Utc::now().naive_utc();
        let start_filter = start_date.unwrap_or_else(|| {
            // Default to 90 days ago
            now - chrono::Duration::days(90)
        });
        let end_filter = end_date.unwrap_or(now);
        (start_filter, end_filter)
    }
}
