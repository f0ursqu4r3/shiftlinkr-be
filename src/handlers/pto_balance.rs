use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::models::{PtoBalanceAdjustmentInput, PtoBalanceUpdateInput};
use crate::database::repositories::pto_balance::PtoBalanceRepository;
use crate::handlers::shared::ApiResponse;
use crate::user_context::AsyncUserContext;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryInput {
    pub company_id: Uuid,
    pub user_id: Option<Uuid>,
    pub limit: Option<i32>,
}

/// Helper function to get company ID or return error
fn get_company_id_or_error(
    user_context: &crate::user_context::UserContext,
) -> Result<Uuid, HttpResponse> {
    user_context.company.as_ref().map(|c| c.id).ok_or_else(|| {
        HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "User does not belong to any company",
        ))
    })
}

/// Helper function to check if user can access another user's data
fn can_access_user_data(
    user_context: &crate::user_context::UserContext,
    target_user_id: Uuid,
) -> bool {
    user_context.is_manager_or_admin() || target_user_id == user_context.user.id
}

/// Get PTO balance for a user
pub async fn get_pto_balance(
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<PtoBalanceRepository>,
    path: Option<web::Path<Uuid>>,
) -> Result<HttpResponse> {
    let user_id = match path {
        Some(path) => {
            let target_user_id = path.into_inner();
            if !can_access_user_data(&user_context, target_user_id) {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "Cannot view other users' PTO balance",
                )));
            }
            target_user_id
        }
        None => user_context.user.id,
    };

    let company_id = match get_company_id_or_error(&user_context) {
        Ok(id) => id,
        Err(err) => return Ok(err),
    };

    match repo.get_balance_for_company(user_id, company_id).await {
        Ok(Some(balance)) => Ok(HttpResponse::Ok().json(ApiResponse::success(balance))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("PTO balance not found")))
        }
        Err(err) => {
            log::error!("Error fetching PTO balance: {}", err);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to fetch PTO balance")))
        }
    }
}

/// Update PTO balance for a user (admins/managers only)
pub async fn update_pto_balance(
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
    update: web::Json<PtoBalanceUpdateInput>,
) -> Result<HttpResponse> {
    // Only admins and managers can update PTO balances
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to update PTO balance",
        )));
    }

    let company_id = match get_company_id_or_error(&user_context) {
        Ok(id) => id,
        Err(err) => return Ok(err),
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
                .json(ApiResponse::<()>::error("Failed to update PTO balance")))
        }
    }
}

/// Adjust PTO balance (admins/managers only)
pub async fn adjust_pto_balance(
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>, // Changed from String to Uuid
    adjustment: web::Json<PtoBalanceAdjustmentInput>,
) -> Result<HttpResponse> {
    // Only admins and managers can adjust PTO balances
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to adjust PTO balance",
        )));
    }

    let company_id = match get_company_id_or_error(&user_context) {
        Ok(id) => id,
        Err(err) => return Ok(err),
    };
    let user_id = path.into_inner();

    match repo
        .adjust_balance_for_company(user_id, company_id, adjustment.into_inner())
        .await
    {
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
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
    query: web::Query<BalanceQueryInput>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    // Users can only view their own history unless they're admins/managers
    if !can_access_user_data(&user_context, user_id) {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Cannot view other users' balance history",
        )));
    }

    match repo.get_balance_history(user_id, query.limit).await {
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
    AsyncUserContext(user_context): AsyncUserContext,
    repo: web::Data<PtoBalanceRepository>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    // Only admins and managers can process accruals
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Insufficient permissions to process PTO accrual",
        )));
    }

    let company_id = match get_company_id_or_error(&user_context) {
        Ok(id) => id,
        Err(err) => return Ok(err),
    };
    let user_id = path.into_inner();

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
                .json(ApiResponse::<()>::error("Failed to process PTO accrual")))
        }
    }
}
