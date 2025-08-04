// TODO: refactor
use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::repositories::stats::StatsRepository;
use crate::handlers::shared::ApiResponse;
use crate::services::user_context::AsyncUserContext;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<StatsRepository>,
    query: web::Query<StatsQuery>,
) -> Result<HttpResponse> {
    // Determine user filter based on permissions
    let user_id = if user_context.is_manager_or_admin() {
        // Admins and managers can query organization-wide stats or specific users
        query
            .user_id
            .as_ref()
            .and_then(|id| id.parse::<Uuid>().ok())
    } else {
        // Employees can only see their own stats
        Some(user_context.user.id)
    };

    match repo
        .get_dashboard_stats_for_user(user_id, query.start_date, query.end_date)
        .await
    {
        Ok(stats) => Ok(ApiResponse::success(stats)),
        Err(err) => {
            log::error!("Error fetching dashboard stats: {}", err);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to fetch dashboard statistics",
                )),
            )
        }
    }
}

/// Get shift statistics
pub async fn get_shift_stats(
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<StatsRepository>,
    query: web::Query<StatsQuery>,
) -> Result<HttpResponse> {
    // Determine user filter based on permissions
    let user_id = if user_context.is_manager_or_admin() {
        // Admins and managers can query organization-wide stats or specific users
        query
            .user_id
            .as_ref()
            .and_then(|id| id.parse::<Uuid>().ok())
    } else {
        // Employees can only see their own stats
        Some(user_context.user.id)
    };

    match repo
        .get_shift_stats(user_id, query.start_date, query.end_date)
        .await
    {
        Ok(stats) => Ok(ApiResponse::success(stats)),
        Err(err) => {
            log::error!("Error fetching shift stats: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch shift statistics")))
        }
    }
}

/// Get time-off statistics
pub async fn get_time_off_stats(
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<StatsRepository>,
    query: web::Query<StatsQuery>,
) -> Result<HttpResponse> {
    // Determine user filter based on permissions
    let user_id = if user_context.is_manager_or_admin() {
        // Admins and managers can query organization-wide stats or specific users
        query
            .user_id
            .as_ref()
            .and_then(|id| id.parse::<Uuid>().ok())
    } else {
        // Employees can only see their own stats
        Some(user_context.user.id)
    };

    match repo
        .get_time_off_stats(user_id, query.start_date, query.end_date)
        .await
    {
        Ok(stats) => Ok(ApiResponse::success(stats)),
        Err(err) => {
            log::error!("Error fetching time-off stats: {}", err);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to fetch time-off statistics",
                )),
            )
        }
    }
}
