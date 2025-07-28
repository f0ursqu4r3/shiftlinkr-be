use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::models::{PtoBalanceAdjustmentInput, PtoBalanceUpdateInput};
use crate::database::repositories::pto_balance::PtoBalanceRepository;
use crate::handlers::shared::ApiResponse;
use crate::services::auth::Claims;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryInput {
    pub company_id: Uuid,
    pub user_id: Option<Uuid>,
    pub limit: Option<i32>,
}

/// Get PTO balance for a user
pub async fn get_pto_balance(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    query: web::Query<BalanceQueryInput>,
) -> Result<HttpResponse> {
    let company_id = query.company_id;

    // Determine which user's balance to retrieve
    let user_id = if let Some(requested_user_id) = query.user_id {
        // Only admins and managers can view other users' balances
        if !claims.is_manager_or_admin() && requested_user_id != claims.sub {
            return Ok(HttpResponse::Forbidden()
                .json(ApiResponse::error("Cannot view other users' balances")));
        }
        requested_user_id
    } else {
        // Default to current user's balance
        claims.sub
    };

    match repo.get_balance_for_company(user_id, company_id).await {
        Ok(Some(balance)) => Ok(HttpResponse::Ok().json(ApiResponse::success(balance))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error("User not found"))),
        Err(err) => {
            log::error!("Error fetching PTO balance: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch PTO balance")))
        }
    }
}

/// Update PTO balance for a user (admins/managers only)
pub async fn update_pto_balance(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
    update: web::Json<PtoBalanceUpdateInput>,
) -> Result<HttpResponse> {
    // Only admins and managers can update PTO balances
    if !claims.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to update PTO balance",
        )));
    }

    let company_id = match claims.company_id {
        Some(id) => {
            if update.company_id != id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
                    "Cannot update PTO balance for a different company",
                )));
            }
            id
        }
        None => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::error(
                "Company ID is required for updating PTO balance",
            )));
        }
    };

    let user_id = path.into_inner();

    match repo
        .update_balance_for_company(user_id, company_id, update.into_inner())
        .await
    {
        Ok(updated_balance) => Ok(HttpResponse::Ok().json(ApiResponse::success(updated_balance))),
        Err(err) => {
            log::error!("Error updating PTO balance: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to update PTO balance")))
        }
    }
}

/// Adjust PTO balance (admins/managers only)
pub async fn adjust_pto_balance(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<String>,
    adjustment: web::Json<PtoBalanceAdjustmentInput>,
) -> Result<HttpResponse> {
    // Only admins and managers can adjust PTO balances
    if !claims.is_admin() && !claims.is_manager() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to adjust PTO balance",
        )));
    }

    let user_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::error("Invalid user ID format")))
        }
    };

    let company_id = adjustment.company_id;

    match repo
        .adjust_balance_for_company(user_id, company_id, adjustment.into_inner())
        .await
    {
        Ok(history) => Ok(HttpResponse::Created().json(ApiResponse::success(history))),
        Err(err) => {
            log::error!("Error adjusting PTO balance: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to adjust PTO balance")))
        }
    }
}

/// Get PTO balance history for a user
pub async fn get_pto_balance_history(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
    query: web::Query<BalanceQueryInput>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only view their own history unless they're admins/managers
    if !claims.is_manager_or_admin() && user_id != claims.sub {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Cannot view other users' balance history",
        )));
    }

    match repo.get_balance_history(user_id, query.limit).await {
        Ok(history) => Ok(HttpResponse::Ok().json(ApiResponse::success(history))),
        Err(err) => {
            log::error!("Error fetching PTO balance history: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to fetch balance history")))
        }
    }
}

/// Process PTO accrual for a user (admins/managers only)
pub async fn process_pto_accrual(
    claims: Claims,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse> {
    // Only admins and managers can process accruals
    if !claims.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error(
            "Insufficient permissions to process PTO accrual",
        )));
    }

    let (user_id, company_id) = path.into_inner();
    match repo.process_accrual_for_company(user_id, company_id).await {
        Ok(Some(history)) => Ok(HttpResponse::Created().json(ApiResponse::success(history))),
        Ok(None) => Ok(
            HttpResponse::Ok().json(ApiResponse::<()>::success_with_message(
                None,
                "No accrual processed",
            )),
        ),
        Err(err) => {
            log::error!("Error processing PTO accrual: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to process PTO accrual")))
        }
    }
}
