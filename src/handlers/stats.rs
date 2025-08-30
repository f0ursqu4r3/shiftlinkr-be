use actix_web::{
    HttpResponse, Result,
    web::{Data, Query},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::repositories::stats as stats_repo,
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::{CacheLayer, cache::InvalidationContext, request_info::RequestInfo},
    services::user_context::UserContext,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    query: Query<StatsQuery>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let target_user_id = query.user_id.or_else(|| Some(user_id)).unwrap();

    ctx.requires_same_user(target_user_id)?;

    let stats =
        stats_repo::get_dashboard_stats_for_user(query.user_id, query.start_date, query.end_date)
            .await
            .map_err(|err| {
                log::error!("Error fetching dashboard stats: {}", err);
                AppError::DatabaseError(err)
            })?;

    // Cache dashboard stats - they're expensive to calculate and frequently accessed
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(target_user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(stats))
}

/// Get shift statistics
pub async fn get_shift_stats(
    query: Query<StatsQuery>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let target_user_id = query.user_id.or_else(|| Some(user_id)).unwrap();

    ctx.requires_same_user(target_user_id)?;

    let stats = stats_repo::get_shift_stats(query.user_id, query.start_date, query.end_date)
        .await
        .map_err(|err| {
            log::error!("Error fetching shift stats: {}", err);
            AppError::DatabaseError(err)
        })?;

    // Cache shift stats - affects shifts and stats resources
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(target_user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(stats))
}

/// Get time-off statistics
pub async fn get_time_off_stats(
    query: Query<StatsQuery>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    let target_user_id = query.user_id.or_else(|| Some(user_id)).unwrap();

    ctx.requires_same_user(target_user_id)?;

    let stats = stats_repo::get_time_off_stats(query.user_id, query.start_date, query.end_date)
        .await
        .map_err(|err| {
            log::error!("Error fetching time-off stats: {}", err);
            AppError::DatabaseError(err)
        })?;

    // Cache time-off stats - affects time-off and stats resources
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(ctx.strict_company_id()?),
                user_id: Some(target_user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(stats))
}
