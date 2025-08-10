use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::repositories::stats as stats_repo;
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::services::user_context::extract_context;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    query: web::Query<StatsQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    // Determine user filter based on permissions
    let user_id = user_context.user_id();

    let target_user_id = query.user_id.or_else(|| Some(user_id)).unwrap();

    user_context.requires_same_user(target_user_id)?;

    let stats =
        stats_repo::get_dashboard_stats_for_user(query.user_id, query.start_date, query.end_date)
            .await
            .map_err(|err| {
                log::error!("Error fetching dashboard stats: {}", err);
                AppError::DatabaseError(err)
            })?;

    Ok(ApiResponse::success(stats))
}

/// Get shift statistics
pub async fn get_shift_stats(
    query: web::Query<StatsQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();

    let target_user_id = query.user_id.or_else(|| Some(user_id)).unwrap();

    user_context.requires_same_user(target_user_id)?;

    let stats = stats_repo::get_shift_stats(query.user_id, query.start_date, query.end_date)
        .await
        .map_err(|err| {
            log::error!("Error fetching shift stats: {}", err);
            AppError::DatabaseError(err)
        })?;

    Ok(ApiResponse::success(stats))
}

/// Get time-off statistics
pub async fn get_time_off_stats(
    query: web::Query<StatsQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_context = extract_context(&req).await?;

    let user_id = user_context.user_id();

    let target_user_id = query.user_id.or_else(|| Some(user_id)).unwrap();

    user_context.requires_same_user(target_user_id)?;

    let stats = stats_repo::get_time_off_stats(query.user_id, query.start_date, query.end_date)
        .await
        .map_err(|err| {
            log::error!("Error fetching time-off stats: {}", err);
            AppError::DatabaseError(err)
        })?;

    Ok(ApiResponse::success(stats))
}
