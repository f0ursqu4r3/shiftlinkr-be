use std::collections::HashMap;

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::database::{
    models::{ActivityType, CreateActivityInput, EntityType},
    repositories::activity as activity_repo,
};
use crate::middleware::request_info::RequestInfo;

/// Generic activity logging for custom cases
pub async fn log_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    activity_type: String,
    entity_type: String,
    entity_id: Uuid,
    action: String,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type,
        entity_type,
        entity_id,
        action,
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

/// Log user management activity
pub async fn log_user_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    target_user_id: Uuid,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::USER_MANAGEMENT.to_string(),
        entity_type: EntityType::USER.to_string(),
        entity_id: target_user_id,
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

/// Log authentication activity
pub async fn log_auth_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::AUTHENTICATION.to_string(),
        entity_type: EntityType::USER.to_string(),
        entity_id: user_id.unwrap_or(Uuid::nil()), // Use nil UUID for failed logins where user_id is unknown
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

/// Log location management activity
pub async fn log_location_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    location_id: Uuid,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::LOCATION_MANAGEMENT.to_string(),
        entity_type: EntityType::LOCATION.to_string(),
        entity_id: location_id,
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

/// Log team management activity
pub async fn log_team_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    team_id: Uuid,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::TEAM_MANAGEMENT.to_string(),
        entity_type: EntityType::TEAM.to_string(),
        entity_id: team_id,
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

/// Log shift management activity
pub async fn log_shift_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    shift_id: Uuid,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::SHIFT_MANAGEMENT.to_string(),
        entity_type: EntityType::SHIFT.to_string(),
        entity_id: shift_id,
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

/// Log time off management activity
pub async fn log_time_off_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    time_off_id: Uuid,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::TIME_OFF_MANAGEMENT.to_string(),
        entity_type: EntityType::TIME_OFF.to_string(),
        entity_id: time_off_id,
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

/// Log shift swap activity
pub async fn log_shift_swap_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    swap_id: Uuid,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::SHIFT_MANAGEMENT.to_string(),
        entity_type: EntityType::SHIFT_SWAP.to_string(),
        entity_id: swap_id,
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

pub async fn log_skill_activity(
    tx: &mut Transaction<'_, Postgres>,
    company_id: Uuid,
    user_id: Option<Uuid>,
    skill_id: Uuid,
    action: &str,
    description: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
    req: &RequestInfo,
) -> Result<(), sqlx::Error> {
    let request = CreateActivityInput {
        company_id,
        user_id,
        activity_type: ActivityType::SKILL_MANAGEMENT.to_string(),
        entity_type: EntityType::SKILL.to_string(),
        entity_id: skill_id,
        action: action.to_string(),
        description,
        metadata,
        ip_address: req.ip_address.clone(),
        user_agent: req.user_agent.clone(),
    };

    activity_repo::log_activity(tx, request).await?;
    Ok(())
}

pub fn metadata(pairs: Vec<(&str, String)>) -> HashMap<String, serde_json::Value> {
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v)))
        .collect()
}
