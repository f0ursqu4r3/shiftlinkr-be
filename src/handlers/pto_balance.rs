use actix_web::{HttpResponse, Result, web::{self, Data}};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        models::{PtoBalanceAdjustmentInput, PtoBalanceUpdateInput},
        repositories::pto_balance as pto_repo,
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::{CacheLayer, cache::InvalidationContext, request_info::RequestInfo},
    services::{activity_logger, user_context::UserContext},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryInput {
    pub user_id: Option<Uuid>,
    pub limit: Option<i32>,
}

/// Get PTO balance for a user
pub async fn get_pto_balance(
    path: Option<web::Path<Uuid>>,
    ctx: UserContext,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let user_id = path.map(|p| p.into_inner()).unwrap_or(ctx.user.id);

    ctx.requires_same_user(user_id)?;

    let company_id = ctx.strict_company_id()?;

    let balance = pto_repo::get_balance_for_company(user_id, company_id)
        .await
        .map_err(|err| {
            log::error!("Error fetching PTO balance: {}", err);
            AppError::DatabaseError(err)
        })?
        .ok_or_else(|| {
            log::error!("PTO balance not found for user {}", user_id);
            AppError::NotFound(format!("PTO balance not found for user {}", user_id))
        })?;

    // Cache PTO balance - affects pto, users resources
    cache
        .invalidate(
            &req_info.path,
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(balance))
}

/// Update PTO balance for a user (admins/managers only)
pub async fn update_pto_balance(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<PtoBalanceUpdateInput>,
    req_info: RequestInfo,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;
    let user_id = path.into_inner();
    let update_data = input.into_inner();
    let path_for_cache = req_info.path.clone();

    let balance = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let balance =
                pto_repo::update_balance_for_company(tx, user_id, company_id, update_data.clone())
                    .await?;

            // Log the update activity
            let metadata = activity_logger::metadata(vec![
                ("company_id", company_id.to_string()),
                ("user_id", user_id.to_string()),
                (
                    "pto_balance_hours",
                    update_data.pto_balance_hours.unwrap_or(0).to_string(),
                ),
                (
                    "sick_balance_hours",
                    update_data.sick_balance_hours.unwrap_or(0).to_string(),
                ),
                (
                    "personal_balance_hours",
                    update_data.personal_balance_hours.unwrap_or(0).to_string(),
                ),
                (
                    "pto_accrual_rate",
                    update_data.pto_accrual_rate.unwrap_or_default().to_string(),
                ),
                (
                    "hire_date",
                    update_data.hire_date.unwrap_or_default().to_string(),
                ),
            ]);

            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(ctx.user.id),
                user_id,
                "update_pto_balance",
                format!(
                    "Updated PTO balance for user {} in company {}",
                    user_id, company_id
                ),
                Some(metadata),
                &req_info,
            )
            .await?;

            Ok(balance)
        })
    })
    .await?;

    // Cache invalidation for PTO balance update - affects pto, users, stats
    cache
        .invalidate(
            &path_for_cache,
            &InvalidationContext {
                company_id: Some(company_id),
                user_id: Some(user_id),
                resource_id: None,
            },
        )
        .await;

    Ok(ApiResponse::success(balance))
}

/// Adjust PTO balance (admins/managers only)
pub async fn adjust_pto_balance(
    path: web::Path<Uuid>,
    ctx: UserContext,
    input: web::Json<PtoBalanceAdjustmentInput>,
    req: RequestInfo,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;
    let user_id = path.into_inner();
    let adjustment_data = input.into_inner();

    let balance = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let balance = pto_repo::adjust_balance_for_company(
                tx,
                user_id,
                company_id,
                adjustment_data.clone(),
            )
            .await?;

            // Log the adjustment activity
            let metadata = activity_logger::metadata(vec![
                ("company_id", company_id.to_string()),
                ("user_id", user_id.to_string()),
                ("change_type", adjustment_data.change_type.to_string()),
                ("balance_type", adjustment_data.balance_type.to_string()),
                ("hours_changed", adjustment_data.hours_changed.to_string()),
                ("description", adjustment_data.description.clone()),
            ]);

            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(ctx.user.id),
                user_id,
                "adjust_pto_balance",
                format!(
                    "Adjusted PTO balance for user {} in company {}",
                    user_id, company_id
                ),
                Some(metadata),
                &req,
            )
            .await?;

            Ok(balance)
        })
    })
    .await?;

    Ok(ApiResponse::success(balance))
}

/// Get PTO balance history for a user
pub async fn get_pto_balance_history(
    path: web::Path<Uuid>,
    query: web::Query<BalanceQueryInput>,
    ctx: UserContext,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let company_id = ctx.strict_company_id()?;

    ctx.requires_same_user(user_id)?;

    let balance_history = pto_repo::get_balance_history(user_id, company_id, query.limit)
        .await
        .map_err(|err| {
            log::error!("Error fetching PTO balance history: {}", err);
            AppError::DatabaseError(err)
        })?;

    Ok(ApiResponse::success(balance_history))
}

/// Process PTO accrual for a user (admins/managers only)
pub async fn process_pto_accrual(
    path: web::Path<Uuid>,
    req: RequestInfo,
    ctx: UserContext,
) -> Result<HttpResponse> {
    ctx.requires_manager()?;

    let company_id = ctx.strict_company_id()?;
    let user_id = path.into_inner();

    let result = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let result = pto_repo::process_accrual_for_company(tx, user_id, company_id)
                .await?
                .ok_or_else(|| {
                    log::error!("PTO accrual not found for user {}", user_id);
                    AppError::NotFound(format!("PTO accrual not found for user {}", user_id))
                })?;

            // Log the accrual activity within the transaction
            let metadata = activity_logger::metadata(vec![
                ("user_id", result.user_id.to_string()),
                ("company_id", result.company_id.to_string()),
                (
                    "hire_date",
                    result
                        .hire_date
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                ),
                (
                    "last_accrual_date",
                    result
                        .last_accrual_date
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                ),
                (
                    "months_since_last_accrual",
                    result.months_since_last_accrual.to_string(),
                ),
                ("hours_to_accrue", result.hours_to_accrue.to_string()),
                ("new_balance", result.new_balance.to_string()),
            ]);

            activity_logger::log_user_activity(
                tx,
                company_id,
                Some(ctx.user.id),
                user_id,
                "process_pto_accrual",
                format!("Processed PTO accrual for user {}", user_id),
                Some(metadata),
                &req,
            )
            .await?;

            Ok(result)
        })
    })
    .await?;

    Ok(ApiResponse::success(result))
}
