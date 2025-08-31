use chrono::{DateTime, Utc};
use sqlx::{Postgres, Result, Row, Transaction};
use uuid::Uuid;

use crate::database::{
    get_pool,
    models::{CompanySubscription, Invoice, PaymentMethod, SubscriptionPlan, SubscriptionWithPlan},
    utils::sql,
};

/// Repository for subscription-related database operations

/// Create a new subscription plan
pub async fn create_plan(
    tx: &mut Transaction<'_, Postgres>,
    stripe_price_id: &str,
    name: &str,
    description: Option<&str>,
    price_cents: i32,
    currency: &str,
    interval: &str,
    interval_count: i32,
    max_users: Option<i32>,
    max_companies: Option<i32>,
    features: Option<serde_json::Value>,
) -> Result<SubscriptionPlan> {
    sqlx::query_as::<_, SubscriptionPlan>(&sql(r#"
            INSERT INTO
                subscription_plans (
                    stripe_price_id,
                    name,
                    description,
                    price_cents,
                    currency,
                    INTERVAL,
                    interval_count,
                    max_users,
                    max_companies,
                    features
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
        "#))
    .bind(stripe_price_id)
    .bind(name)
    .bind(description)
    .bind(price_cents)
    .bind(currency)
    .bind(interval)
    .bind(interval_count)
    .bind(max_users)
    .bind(max_companies)
    .bind(features)
    .fetch_one(&mut **tx)
    .await
}

/// Get subscription plan by ID
pub async fn get_plan_by_id(plan_id: Uuid) -> Result<SubscriptionPlan> {
    sqlx::query_as::<_, SubscriptionPlan>(&sql("SELECT * FROM subscription_plans WHERE id = ?"))
        .bind(plan_id)
        .fetch_one(&get_pool().await)
        .await
}

/// Get subscription plan by Stripe price ID
pub async fn get_plan_by_stripe_price_id(stripe_price_id: &str) -> Result<SubscriptionPlan> {
    sqlx::query_as::<_, SubscriptionPlan>(&sql(
        "SELECT * FROM subscription_plans WHERE stripe_price_id = ?",
    ))
    .bind(stripe_price_id)
    .fetch_one(&get_pool().await)
    .await
}

/// Get all active subscription plans
pub async fn get_active_plans() -> Result<Vec<SubscriptionPlan>> {
    sqlx::query_as::<_, SubscriptionPlan>(&sql(
        "SELECT * FROM subscription_plans WHERE is_active = true ORDER BY price_cents ASC",
    ))
    .fetch_all(&get_pool().await)
    .await
}

/// Create a company subscription
pub async fn create_company_subscription(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    stripe_customer_id: &str,
    subscription_plan_id: Option<Uuid>,
) -> Result<CompanySubscription> {
    sqlx::query_as::<_, CompanySubscription>(&sql(r#"
            INSERT INTO
                company_subscriptions (company_id, stripe_customer_id, subscription_plan_id)
            VALUES
                (?, ?, ?)
            RETURNING *
        "#))
    .bind(company_id)
    .bind(stripe_customer_id)
    .bind(subscription_plan_id)
    .fetch_one(&mut **tx)
    .await
}

/// Get company subscription by company ID
pub async fn get_company_subscription(company_id: Uuid) -> Result<CompanySubscription> {
    sqlx::query_as::<_, CompanySubscription>(&sql(
        "SELECT * FROM company_subscriptions WHERE company_id = ?",
    ))
    .bind(company_id)
    .fetch_one(&get_pool().await)
    .await
}

/// Get company subscription with plan details
pub async fn get_company_subscription_with_plan(company_id: Uuid) -> Result<SubscriptionWithPlan> {
    let row = sqlx::query(&sql(r#"
            SELECT
                cs.*,
                sp.id as plan_id,
                sp.stripe_price_id,
                sp.name as plan_name,
                sp.description as plan_description,
                sp.price_cents,
                sp.currency,
                sp.interval,
                sp.interval_count,
                sp.is_active,
                sp.max_users,
                sp.max_companies,
                sp.features,
                sp.created_at as plan_created_at,
                sp.updated_at as plan_updated_at
            FROM company_subscriptions cs
            LEFT JOIN subscription_plans sp ON cs.subscription_plan_id = sp.id
            WHERE cs.company_id = ?
        "#))
    .bind(company_id)
    .fetch_one(&get_pool().await)
    .await?;

    let plan_id: Option<Uuid> = row.try_get("plan_id")?;
    let plan = if plan_id.is_some() {
        Some(SubscriptionPlan {
            id: plan_id.unwrap(),
            stripe_price_id: row.get("stripe_price_id"),
            name: row.get("plan_name"),
            description: row.try_get("plan_description")?,
            price_cents: row.get("price_cents"),
            currency: row.get("currency"),
            interval: row.get("interval"),
            interval_count: row.get("interval_count"),
            is_active: row.get("is_active"),
            max_users: row.try_get("max_users")?,
            max_companies: row.get("max_companies"),
            features: row.try_get("features")?,
            created_at: row.get("plan_created_at"),
            updated_at: row.get("plan_updated_at"),
        })
    } else {
        None
    };

    Ok(SubscriptionWithPlan {
        subscription: CompanySubscription {
            id: row.get("id"),
            company_id: row.get("company_id"),
            stripe_customer_id: row.get("stripe_customer_id"),
            stripe_subscription_id: row.try_get("stripe_subscription_id")?,
            subscription_plan_id: row.try_get("subscription_plan_id")?,
            status: row.get::<String, _>("status").parse().unwrap_or_default(),
            current_period_start: row.try_get("current_period_start")?,
            current_period_end: row.try_get("current_period_end")?,
            cancel_at_period_end: row.get("cancel_at_period_end"),
            canceled_at: row.try_get("canceled_at")?,
            trial_start: row.try_get("trial_start")?,
            trial_end: row.try_get("trial_end")?,
            price_cents: row.try_get("price_cents")?,
            currency: row.try_get("currency")?,
            interval: row.try_get("interval")?,
            interval_count: row.try_get("interval_count")?,
            metadata: row.try_get("metadata")?,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        },
        plan,
    })
}

/// Update company subscription
pub async fn update_company_subscription(
    tx: &mut Transaction<'_, Postgres>,
    subscription_id: Uuid,
    stripe_subscription_id: Option<&str>,
    status: Option<&str>,
    current_period_start: Option<DateTime<Utc>>,
    current_period_end: Option<DateTime<Utc>>,
    cancel_at_period_end: Option<bool>,
    canceled_at: Option<DateTime<Utc>>,
    trial_start: Option<DateTime<Utc>>,
    trial_end: Option<DateTime<Utc>>,
    metadata: Option<serde_json::Value>,
) -> Result<CompanySubscription> {
    sqlx::query_as::<_, CompanySubscription>(&sql(r#"
            UPDATE company_subscriptions
            SET
                stripe_subscription_id = COALESCE(?, stripe_subscription_id),
                status = COALESCE(?, status),
                current_period_start = COALESCE(?, current_period_start),
                current_period_end = COALESCE(?, current_period_end),
                cancel_at_period_end = COALESCE(?, cancel_at_period_end),
                canceled_at = COALESCE(?, canceled_at),
                trial_start = COALESCE(?, trial_start),
                trial_end = COALESCE(?, trial_end),
                metadata = COALESCE(?, metadata),
                updated_at = NOW ()
            WHERE
                id = ?
            RETURNING *
        "#))
    .bind(stripe_subscription_id)
    .bind(status)
    .bind(current_period_start)
    .bind(current_period_end)
    .bind(cancel_at_period_end)
    .bind(canceled_at)
    .bind(trial_start)
    .bind(trial_end)
    .bind(metadata)
    .bind(subscription_id)
    .fetch_one(&mut **tx)
    .await
}

/// Create payment method
pub async fn create_payment_method(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    stripe_payment_method_id: &str,
    r#type: &str,
    last4: Option<&str>,
    brand: Option<&str>,
    expiry_month: Option<i32>,
    expiry_year: Option<i32>,
    is_default: bool,
    metadata: Option<serde_json::Value>,
) -> Result<PaymentMethod> {
    sqlx::query_as::<_, PaymentMethod>(&sql(r#"
            INSERT INTO
                payment_methods (
                    company_id,
                    stripe_payment_method_id,
                    type,
                    last4,
                    brand,
                    expiry_month,
                    expiry_year,
                    is_default,
                    metadata
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
        "#))
    .bind(company_id)
    .bind(stripe_payment_method_id)
    .bind(r#type)
    .bind(last4)
    .bind(brand)
    .bind(expiry_month)
    .bind(expiry_year)
    .bind(is_default)
    .bind(metadata)
    .fetch_one(&mut **tx)
    .await
}

/// Get company's payment methods
pub async fn get_company_payment_methods(company_id: Uuid) -> Result<Vec<PaymentMethod>> {
    sqlx::query_as::<_, PaymentMethod>(&sql(r#"
            SELECT
                *
            FROM
                payment_methods
            WHERE
                company_id = ?
            ORDER BY
                is_default DESC,
                created_at DESC
        "#))
    .bind(company_id)
    .fetch_all(&get_pool().await)
    .await
}

/// Create invoice
pub async fn create_invoice(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    stripe_invoice_id: &str,
    subscription_id: Option<Uuid>,
    amount_due: i32,
    amount_paid: i32,
    currency: &str,
    status: &str,
    invoice_pdf: Option<&str>,
    hosted_invoice_url: Option<&str>,
    period_start: Option<DateTime<Utc>>,
    period_end: Option<DateTime<Utc>>,
    due_date: Option<DateTime<Utc>>,
    paid_at: Option<DateTime<Utc>>,
    metadata: Option<serde_json::Value>,
) -> Result<Invoice> {
    sqlx::query_as::<_, Invoice>(&sql(r#"
            INSERT INTO
                invoices (
                    company_id,
                    stripe_invoice_id,
                    subscription_id,
                    amount_due,
                    amount_paid,
                    currency,
                    status,
                    invoice_pdf,
                    hosted_invoice_url,
                    period_start,
                    period_end,
                    due_date,
                    paid_at,
                    metadata
                )
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
        "#))
    .bind(company_id)
    .bind(stripe_invoice_id)
    .bind(subscription_id)
    .bind(amount_due)
    .bind(amount_paid)
    .bind(currency)
    .bind(status)
    .bind(invoice_pdf)
    .bind(hosted_invoice_url)
    .bind(period_start)
    .bind(period_end)
    .bind(due_date)
    .bind(paid_at)
    .bind(metadata)
    .fetch_one(&mut **tx)
    .await
}

/// Get company's invoices
pub async fn get_company_invoices(company_id: Uuid) -> Result<Vec<Invoice>> {
    sqlx::query_as::<_, Invoice>(&sql(r#"
            SELECT * FROM invoices WHERE company_id = ? ORDER BY created_at DESC
        "#))
    .bind(company_id)
    .fetch_all(&get_pool().await)
    .await
}

/// Update company subscription status
pub async fn update_company_subscription_status(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    status: &str,
    subscription_plan_id: Option<Uuid>,
    trial_ends_at: Option<DateTime<Utc>>,
    subscription_ends_at: Option<DateTime<Utc>>,
) -> Result<()> {
    sqlx::query(&sql(r#"
            UPDATE companies
            SET
                subscription_status = ?,
                subscription_plan_id = ?,
                trial_ends_at = ?,
                subscription_ends_at = ?,
                updated_at = NOW()
            WHERE id = ?
        "#))
    .bind(status)
    .bind(subscription_plan_id)
    .bind(trial_ends_at)
    .bind(subscription_ends_at)
    .bind(company_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
