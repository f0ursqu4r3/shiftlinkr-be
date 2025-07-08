use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use chrono::NaiveDateTime;

use crate::database::stats_repository::StatsRepository;
use crate::auth::Claims;
use crate::handlers::admin::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub user_id: Option<String>,
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    claims: Claims,
    repo: web::Data<StatsRepository>,
    query: web::Query<StatsQuery>,
) -> Result<HttpResponse> {
    // Determine user filter based on permissions
    let user_id = if claims.is_admin() || claims.is_manager() {
        // Admins and managers can query organization-wide stats or specific users
        query.user_id.as_deref()
    } else {
        // Employees can only see their own stats
        Some(claims.sub.as_str())
    };

    match repo.get_dashboard_stats(user_id, query.start_date, query.end_date).await {
        Ok(stats) => Ok(HttpResponse::Ok().json(ApiResponse::success(stats))),
        Err(err) => {
            log::error!("Error fetching dashboard stats: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch dashboard statistics")))
        }
    }
}

/// Get shift statistics
pub async fn get_shift_stats(
    claims: Claims,
    repo: web::Data<StatsRepository>,
    query: web::Query<StatsQuery>,
) -> Result<HttpResponse> {
    // Determine user filter based on permissions
    let user_id = if claims.is_admin() || claims.is_manager() {
        // Admins and managers can query organization-wide stats or specific users
        query.user_id.as_deref()
    } else {
        // Employees can only see their own stats
        Some(claims.sub.as_str())
    };

    match repo.get_shift_stats(user_id, query.start_date, query.end_date).await {
        Ok(stats) => Ok(HttpResponse::Ok().json(ApiResponse::success(stats))),
        Err(err) => {
            log::error!("Error fetching shift stats: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch shift statistics")))
        }
    }
}

/// Get time-off statistics
pub async fn get_time_off_stats(
    claims: Claims,
    repo: web::Data<StatsRepository>,
    query: web::Query<StatsQuery>,
) -> Result<HttpResponse> {
    // Determine user filter based on permissions
    let user_id = if claims.is_admin() || claims.is_manager() {
        // Admins and managers can query organization-wide stats or specific users
        query.user_id.as_deref()
    } else {
        // Employees can only see their own stats
        Some(claims.sub.as_str())
    };

    match repo.get_time_off_stats(user_id, query.start_date, query.end_date).await {
        Ok(stats) => Ok(HttpResponse::Ok().json(ApiResponse::success(stats))),
        Err(err) => {
            log::error!("Error fetching time-off stats: {}", err);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to fetch time-off statistics")))
        }
    }
}
