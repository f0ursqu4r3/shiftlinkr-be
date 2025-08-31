use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::macros::string_enum;

/// Subscription plan model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub stripe_price_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub currency: String,
    pub interval: String,
    pub interval_count: i32,
    pub is_active: bool,
    pub max_users: Option<i32>,
    pub max_companies: i32,
    pub features: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Company subscription model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanySubscription {
    pub id: Uuid,
    pub company_id: Uuid,
    pub stripe_customer_id: String,
    pub stripe_subscription_id: Option<String>,
    pub subscription_plan_id: Option<Uuid>,
    pub status: SubscriptionStatus,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub price_cents: Option<i32>,
    pub currency: Option<String>,
    pub interval: Option<String>,
    pub interval_count: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription status enum
string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SubscriptionStatus {
        Incomplete => "incomplete",
        IncompleteExpired => "incomplete_expired",
        Trialing => "trialing",
        Active => "active",
        PastDue => "past_due",
        Canceled => "canceled",
        Unpaid => "unpaid",
        Paused => "paused",
    }
}

impl Default for SubscriptionStatus {
    fn default() -> Self {
        SubscriptionStatus::Incomplete
    }
}

/// Payment method model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod {
    pub id: Uuid,
    pub company_id: Uuid,
    pub stripe_payment_method_id: String,
    pub r#type: String,
    pub last4: Option<String>,
    pub brand: Option<String>,
    pub expiry_month: Option<i32>,
    pub expiry_year: Option<i32>,
    pub is_default: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Invoice model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub stripe_invoice_id: String,
    pub subscription_id: Option<Uuid>,
    pub amount_due: i32,
    pub amount_paid: i32,
    pub currency: String,
    pub status: InvoiceStatus,
    pub invoice_pdf: Option<String>,
    pub hosted_invoice_url: Option<String>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Invoice status enum
string_enum! {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum InvoiceStatus {
        Draft => "draft",
        Open => "open",
        Paid => "paid",
        Void => "void",
        Uncollectible => "uncollectible",
    }
}

/// Input structs for creating subscriptions
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionPlanInput {
    pub stripe_price_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub currency: String,
    pub interval: String,
    pub interval_count: i32,
    pub max_users: Option<i32>,
    pub max_companies: Option<i32>,
    pub features: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCompanySubscriptionInput {
    pub company_id: Uuid,
    pub stripe_customer_id: String,
    pub subscription_plan_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionInput {
    pub status: Option<SubscriptionStatus>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: Option<bool>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Subscription with plan details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionWithPlan {
    #[serde(flatten)]
    pub subscription: CompanySubscription,
    pub plan: Option<SubscriptionPlan>,
}

/// Company with subscription status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyWithSubscription {
    #[serde(flatten)]
    pub company: super::Company,
    pub subscription_status: Option<SubscriptionStatus>,
    pub subscription_plan: Option<SubscriptionPlan>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub subscription_ends_at: Option<DateTime<Utc>>,
    pub max_users: Option<i32>,
    pub is_active: bool,
}

/// User company with subscription status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCompanyWithSubscription {
    #[serde(flatten)]
    pub user_company: super::UserCompany,
    pub subscription_status: Option<SubscriptionStatus>,
    pub subscription_plan: Option<SubscriptionPlan>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub subscription_ends_at: Option<DateTime<Utc>>,
    pub is_owner: bool,
}
