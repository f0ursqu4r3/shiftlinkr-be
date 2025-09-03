use anyhow::Result;
use sqlx::{Postgres, Transaction};

use crate::database::{
    models::{CompanyActivity, CreateActivityInput},
    utils::sql,
};

/// Log a new activity  
pub async fn log_activity(
    tx: &mut Transaction<'_, Postgres>,
    request: CreateActivityInput,
) -> Result<CompanyActivity, sqlx::Error> {
    let metadata_json = request
        .metadata
        .map(|m| serde_json::to_value(&m).unwrap_or_default());

    let company_activity = sqlx::query_as::<_, CompanyActivity>(&sql(r#"
        INSERT INTO
            company_activity (
                company_id,
                user_id,
                activity_type,
                entity_type,
                entity_id,
                action,
                description,
                metadata,
                ip_address,
                user_agent
            )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING
            id,
            company_id,
            user_id,
            activity_type,
            entity_type,
            entity_id,
            action,
            description,
            metadata,
            ip_address,
            user_agent,
            created_at
    "#))
    .bind(request.company_id)
    .bind(request.user_id)
    .bind(request.activity_type)
    .bind(request.entity_type)
    .bind(request.entity_id)
    .bind(request.action)
    .bind(request.description)
    .bind(metadata_json)
    .bind(request.ip_address)
    .bind(request.user_agent)
    .fetch_one(&mut **tx)
    .await?;

    Ok(company_activity)
}
