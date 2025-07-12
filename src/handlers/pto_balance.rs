use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;

use crate::database::models::{PtoBalanceAdjustment, PtoBalanceUpdate};
use crate::database::repositories::pto_balance::PtoBalanceRepository;
use crate::handlers::admin::ApiResponse;
use crate::services::auth::Claims;

#[derive(Debug, Deserialize)]
pub struct BalanceQuery {
    pub user_id: Option<String>,
    pub limit: Option<i32>,
}

/// Get PTO balance for a user
pub async fn get_pto_balance(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    query: web::Query<BalanceQuery>,
) -> Result<HttpResponse> {
    // Determine which user's balance to retrieve
    let user_id = if let Some(requested_user_id) = &query.user_id {
        // Only admins and managers can view other users' balances
        if !claims.is_admin() && !claims.is_manager() && requested_user_id != &claims.sub {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "Cannot view other users' balances",
            )));
        }
        requested_user_id.as_str()
    } else {
        // Default to current user's balance
        claims.sub.as_str()
    };

    match repo.get_balance(user_id).await {
        Ok(Some(balance)) => Ok(HttpResponse::Ok().json(ApiResponse::success(balance))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found"))),
        Err(err) => {
            log::error!("Error fetching PTO balance: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch PTO balance")))
        }
    }
}

/// Update PTO balance for a user (admins/managers only)
pub async fn update_pto_balance(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<String>,
    update: web::Json<PtoBalanceUpdate>,
) -> Result<HttpResponse> {
    // Only admins and managers can update PTO balances
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to update PTO balance",
        )));
    }

    let user_id = path.into_inner();

    match repo.update_balance(&user_id, update.into_inner()).await {
        Ok(updated_balance) => Ok(HttpResponse::Ok().json(ApiResponse::success(updated_balance))),
        Err(err) => {
            log::error!("Error updating PTO balance: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to update PTO balance")))
        }
    }
}

/// Adjust PTO balance (admins/managers only)
pub async fn adjust_pto_balance(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<String>,
    adjustment: web::Json<PtoBalanceAdjustment>,
) -> Result<HttpResponse> {
    // Only admins and managers can adjust PTO balances
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to adjust PTO balance",
        )));
    }

    let user_id = path.into_inner();

    match repo.adjust_balance(&user_id, adjustment.into_inner()).await {
        Ok(history) => Ok(HttpResponse::Created().json(ApiResponse::success(history))),
        Err(err) => {
            log::error!("Error adjusting PTO balance: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to adjust PTO balance")))
        }
    }
}

/// Get PTO balance history for a user
pub async fn get_pto_balance_history(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<String>,
    query: web::Query<BalanceQuery>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only view their own history unless they're admins/managers
    if !claims.is_admin() && !claims.is_manager() && user_id != claims.sub {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Cannot view other users' balance history",
        )));
    }

    match repo.get_balance_history(&user_id, query.limit).await {
        Ok(history) => Ok(HttpResponse::Ok().json(ApiResponse::success(history))),
        Err(err) => {
            log::error!("Error fetching PTO balance history: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch balance history")))
        }
    }
}

/// Process PTO accrual for a user (admins/managers only)
pub async fn process_pto_accrual(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // Only admins and managers can process accruals
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to process PTO accrual",
        )));
    }

    let user_id = path.into_inner();

    match repo.process_accrual(&user_id).await {
        Ok(Some(history)) => Ok(HttpResponse::Created().json(ApiResponse::success(history))),
        Ok(None) => Ok(
            HttpResponse::Ok().json(ApiResponse::<()>::success_with_message(
                "No accrual processed",
            )),
        ),
        Err(err) => {
            log::error!("Error processing PTO accrual: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to process PTO accrual")))
        }
    }
}
