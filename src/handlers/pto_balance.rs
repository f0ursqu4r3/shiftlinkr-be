use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::models::{PtoBalanceAdjustmentInput, PtoBalanceUpdateInput};
use crate::database::repositories::pto_balance::PtoBalanceRepository;
use crate::error::AppError;
use crate::handlers::shared::ApiResponse;
use crate::user_context::AsyncUserContext;
use crate::ActivityLogger;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryInput {
    pub user_id: Option<Uuid>,
    pub limit: Option<i32>,
}

/// Get PTO balance for a user
pub async fn get_pto_balance(
    AsyncUserContext(user_context): AsyncUserContext,
    pto_repo: web::Data<PtoBalanceRepository>,
    path: Option<web::Path<Uuid>>,
) -> Result<HttpResponse> {
    let user_id = path.map(|p| p.into_inner()).unwrap_or(user_context.user.id);

    user_context.requires_same_user(user_id)?;

    let company_id = user_context.strict_company_id()?;

    let balance = pto_repo
        .get_balance_for_company(user_id, company_id)
        .await
        .map_err(|err| {
            log::error!("Error fetching PTO balance: {}", err);
            AppError::DatabaseError(err)
        })?
        .ok_or_else(|| {
            log::error!("PTO balance not found for user {}", user_id);
            AppError::NotFound(format!("PTO balance not found for user {}", user_id))
        })?;

    Ok(ApiResponse::success(balance))
}

/// Update PTO balance for a user (admins/managers only)
pub async fn update_pto_balance(
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: web::Data<ActivityLogger>,
    pto_repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
    update: web::Json<PtoBalanceUpdateInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;
    let user_id = path.into_inner();
    let update_data = update.into_inner();

    let balance = pto_repo
        .update_balance_for_company(user_id, company_id, update_data.clone())
        .await
        .map_err(|err| {
            log::error!("Error updating PTO balance: {}", err);
            AppError::DatabaseError(err)
        })?;

    // Log the update activity
    let metadata = ActivityLogger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"user_id".to_string(), user_id.to_string()),
        (
            &"pto_balance_hours".to_string(),
            update_data.pto_balance_hours.unwrap_or(0).to_string(),
        ),
        (
            &"sick_balance_hours".to_string(),
            update_data.sick_balance_hours.unwrap_or(0).to_string(),
        ),
        (
            &"personal_balance_hours".to_string(),
            update_data.personal_balance_hours.unwrap_or(0).to_string(),
        ),
        (
            &"pto_accrual_rate".to_string(),
            update_data.pto_accrual_rate.unwrap_or_default().to_string(),
        ),
        (
            &"hire_date".to_string(),
            update_data.hire_date.unwrap_or_default().to_string(),
        ),
    ]);

    if let Err(e) = activity_logger
        .log_user_activity(
            company_id,
            Some(user_context.user.id),
            user_id,
            "update_pto_balance",
            format!(
                "Updated PTO balance for user {} in company {}",
                user_id, company_id
            ),
            Some(metadata),
            &req,
        )
        .await
    {
        log::warn!("Failed to log PTO balance update activity: {}", e);
    }

    Ok(ApiResponse::success(balance))
}

/// Adjust PTO balance (admins/managers only)
pub async fn adjust_pto_balance(
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: web::Data<ActivityLogger>,
    pto_repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>, // Changed from String to Uuid
    adjustment: web::Json<PtoBalanceAdjustmentInput>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Only admins and managers can adjust PTO balances
    user_context.requires_manager()?;

    let company_id = user_context.strict_company_id()?;
    let user_id = path.into_inner();
    let adjustment_data = adjustment.into_inner();

    let balance = pto_repo
        .adjust_balance_for_company(user_id, company_id, adjustment_data.clone())
        .await
        .map_err(|err| {
            log::error!("Error adjusting PTO balance: {}", err);
            AppError::DatabaseError(err)
        })?;

    // Log the adjustment activity
    let metadata = ActivityLogger::metadata(vec![
        (&"company_id".to_string(), company_id.to_string()),
        (&"user_id".to_string(), user_id.to_string()),
        (
            &"change_type".to_string(),
            adjustment_data.change_type.to_string(),
        ),
        (
            &"balance_type".to_string(),
            adjustment_data.balance_type.to_string(),
        ),
        (
            &"hours_changed".to_string(),
            adjustment_data.hours_changed.to_string(),
        ),
        (&"description".to_string(), adjustment_data.description),
    ]);

    if let Err(e) = activity_logger
        .log_user_activity(
            company_id,
            Some(user_context.user.id),
            user_id,
            "adjust_pto_balance",
            format!(
                "Adjusted PTO balance for user {} in company {}",
                user_id, company_id
            ),
            Some(metadata),
            &req,
        )
        .await
    {
        log::warn!("Failed to log PTO balance adjustment activity: {}", e);
    }

    Ok(ApiResponse::success(balance))
}

/// Get PTO balance history for a user
pub async fn get_pto_balance_history(
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
    query: web::Query<BalanceQueryInput>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    user_context.requires_same_user(user_id)?;
    let company_id = user_context.strict_company_id()?;

    let balance_history = repo
        .get_balance_history(user_id, company_id, query.limit)
        .await
        .map_err(|err| {
            log::error!("Error fetching PTO balance history: {}", err);
            AppError::DatabaseError(err)
        })?;

    Ok(ApiResponse::success(balance_history))
}

/// Process PTO accrual for a user (admins/managers only)
pub async fn process_pto_accrual(
    AsyncUserContext(user_context): AsyncUserContext,
    activity_logger: web::Data<ActivityLogger>,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Only admins and managers can process accruals
    user_context.requires_manager()?;
    let company_id = user_context.strict_company_id()?;
    let user_id = path.into_inner();

    let result = repo
        .process_accrual_for_company(user_id, company_id)
        .await
        .map_err(|err| {
            log::error!("Error processing PTO accrual: {}", err);
            AppError::DatabaseError(err)
        })?
        .ok_or_else(|| {
            log::error!("PTO accrual not found for user {}", user_id);
            AppError::NotFound(format!("PTO accrual not found for user {}", user_id))
        })?;

    // Log the accrual activity
    let metadata = ActivityLogger::metadata(vec![
        (&"user_id", result.user_id.to_string()),
        (&"company_id", result.company_id.to_string()),
        (
            &"hire_date",
            result
                .hire_date
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
        ),
        (
            &"last_accrual_date",
            result
                .last_accrual_date
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
        ),
        (
            &"months_since_last_accrual",
            result.months_since_last_accrual.to_string(),
        ),
        (&"hours_to_accrue", result.hours_to_accrue.to_string()),
        (&"new_balance", result.new_balance.to_string()),
    ]);
    if let Err(e) = activity_logger
        .log_user_activity(
            company_id,
            Some(user_context.user.id),
            user_id,
            "process_pto_accrual",
            format!("Processed PTO accrual for user {}", user_id),
            Some(metadata),
            &req,
        )
        .await
    {
        log::warn!("Failed to log PTO accrual activity: {}", e);
    }

    Ok(ApiResponse::success(result))
}
