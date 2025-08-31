use actix_web::{
    HttpResponse, Result,
    web::{Data, Json, Path},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        models::{CompanySubscription, SubscriptionPlan, SubscriptionWithPlan},
        repositories::subscription as subscription_repo,
        transaction::DatabaseTransaction,
    },
    error::AppError,
    handlers::shared::ApiResponse,
    middleware::{CacheLayer, cache::InvalidationContext},
    services::user_context::UserContext,
};

/// Get all active subscription plans
pub async fn get_subscription_plans() -> Result<HttpResponse> {
    // This would typically come from a database query
    // For now, return a placeholder response
    let plans = vec![
        SubscriptionPlan {
            id: Uuid::new_v4(),
            stripe_price_id: "price_basic".to_string(),
            name: "Basic Plan".to_string(),
            description: Some("Perfect for small teams".to_string()),
            price_cents: 9900, // $99.00
            currency: "usd".to_string(),
            interval: "month".to_string(),
            interval_count: 1,
            is_active: true,
            max_users: Some(10),
            max_companies: 1,
            features: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        SubscriptionPlan {
            id: Uuid::new_v4(),
            stripe_price_id: "price_pro".to_string(),
            name: "Professional Plan".to_string(),
            description: Some("For growing businesses".to_string()),
            price_cents: 19900, // $199.00
            currency: "usd".to_string(),
            interval: "month".to_string(),
            interval_count: 1,
            is_active: true,
            max_users: Some(50),
            max_companies: 3,
            features: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    Ok(ApiResponse::success(plans))
}

pub async fn get_company_subscription(ctx: UserContext, path: Path<Uuid>) -> Result<HttpResponse> {
    let company_id = path.into_inner();

    ctx.requires_same_company(company_id)?;

    // Verify user has access to this company
    ctx.requires_same_company(company_id)?;

    match subscription_repo::get_company_subscription_with_plan(company_id).await {
        Ok(subscription) => Ok(ApiResponse::success(subscription)),
        Err(sqlx::Error::RowNotFound) => {
            // Company doesn't have a subscription yet
            Ok(ApiResponse::success(SubscriptionWithPlan {
                subscription: CompanySubscription {
                    id: Uuid::new_v4(),
                    company_id,
                    stripe_customer_id: "".to_string(),
                    stripe_subscription_id: None,
                    subscription_plan_id: None,
                    status: crate::database::models::SubscriptionStatus::Incomplete,
                    current_period_start: None,
                    current_period_end: None,
                    cancel_at_period_end: false,
                    canceled_at: None,
                    trial_start: None,
                    trial_end: None,
                    price_cents: None,
                    currency: None,
                    interval: None,
                    interval_count: None,
                    metadata: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
                plan: None,
            }))
        }
        Err(err) => Err(AppError::DatabaseError(err).into()),
    }
}

/// Create or update company subscription
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionRequest {
    pub stripe_price_id: String,
    pub payment_method_id: Option<String>,
}

pub async fn create_subscription(
    ctx: UserContext,
    path: Path<Uuid>,
    req: Json<CreateSubscriptionRequest>,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let company_id = path.into_inner();
    let user_id = ctx.user_id();

    // Verify user has access to this company
    ctx.requires_same_company(company_id)?;

    // Get the plan details
    let plan = subscription_repo::get_plan_by_stripe_price_id(&req.stripe_price_id)
        .await
        .map_err(|err| AppError::DatabaseError(err))?;

    // Create or update subscription
    let subscription = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let subscription = match subscription_repo::get_company_subscription(company_id).await {
                Ok(existing) => {
                    // Update existing subscription
                    subscription_repo::update_company_subscription(
                        tx,
                        existing.id,
                        None, // Will be set by Stripe webhook
                        Some("incomplete"),
                        None,
                        None,
                        Some(false),
                        None,
                        None,
                        None,
                        None,
                    )
                    .await
                    .map_err(AppError::DatabaseError)?
                }
                Err(sqlx::Error::RowNotFound) => {
                    // Create new subscription
                    subscription_repo::create_company_subscription(
                        tx,
                        company_id,
                        &format!("cus_{}", company_id.simple()), // Temporary customer ID
                        Some(plan.id),
                    )
                    .await
                    .map_err(AppError::DatabaseError)?
                }
                Err(err) => return Err(AppError::DatabaseError(err).into()),
            };

            // Update company's subscription status
            subscription_repo::update_company_subscription_status(
                tx,
                company_id,
                "incomplete",
                Some(plan.id),
                None,
                None,
            )
            .await
            .map_err(AppError::DatabaseError)?;

            Ok(subscription)
        })
    })
    .await?;

    // Invalidate cache
    cache
        .invalidate(
            "subscriptions",
            &InvalidationContext {
                user_id: Some(user_id),
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success(subscription))
}

pub async fn cancel_subscription(
    ctx: UserContext,
    path: Path<Uuid>,
    cache: Data<CacheLayer>,
) -> Result<HttpResponse> {
    let company_id = path.into_inner();
    let user_id = ctx.user_id();

    // Verify user has access to this company
    ctx.requires_same_company(company_id)?;

    let subscription = subscription_repo::get_company_subscription(company_id)
        .await
        .map_err(|err| AppError::DatabaseError(err))?;

    // Update subscription to cancel at period end
    let updated_subscription = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let updated_subscription = subscription_repo::update_company_subscription(
                tx,
                subscription.id,
                None,
                Some("active"), // Keep active until period end
                None,
                None,
                Some(true), // Cancel at period end
                None,
                None,
                None,
                None,
            )
            .await?;

            // Update company's subscription status
            subscription_repo::update_company_subscription_status(
                tx,
                company_id,
                "canceled",
                subscription.subscription_plan_id,
                None,
                subscription.current_period_end,
            )
            .await?;

            Ok(updated_subscription)
        })
    })
    .await?;

    // Invalidate cache
    cache
        .invalidate(
            "subscriptions",
            &InvalidationContext {
                user_id: Some(user_id),
                company_id: Some(company_id),
                ..Default::default()
            },
        )
        .await;

    Ok(ApiResponse::success(updated_subscription))
}

pub async fn get_payment_methods(ctx: UserContext, path: Path<Uuid>) -> Result<HttpResponse> {
    let company_id = path.into_inner();
    // Verify user has access to this company
    ctx.requires_same_company(company_id)?;

    let payment_methods = subscription_repo::get_company_payment_methods(company_id)
        .await
        .map_err(|err| AppError::DatabaseError(err))?;

    Ok(ApiResponse::success(payment_methods))
}

pub async fn get_invoices(ctx: UserContext, path: Path<Uuid>) -> Result<HttpResponse> {
    let company_id = path.into_inner();
    // Verify user has access to this company
    ctx.requires_same_company(company_id)?;

    let invoices = subscription_repo::get_company_invoices(company_id)
        .await
        .map_err(|err| AppError::DatabaseError(err))?;

    Ok(ApiResponse::success(invoices))
}

/// Check if user is an owner (has active subscription)
pub async fn check_owner_status(ctx: UserContext) -> Result<HttpResponse> {
    let user_id = ctx.user_id();

    // TODO: Implement ownership checking logic
    // For now, return false
    let is_owner = false;

    Ok(ApiResponse::success(serde_json::json!({
        "isOwner": is_owner,
        "userId": user_id
    })))
}
