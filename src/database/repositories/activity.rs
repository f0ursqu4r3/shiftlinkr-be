use crate::database::models::{CompanyActivity, CreateActivityInput};
use sqlx::{PgPool, Result};

#[derive(Clone)]
pub struct ActivityRepository {
    pool: PgPool,
}

impl ActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Log a new activity  
    pub async fn log_activity(&self, request: CreateActivityInput) -> Result<CompanyActivity> {
        let metadata_json = request
            .metadata
            .map(|m| serde_json::to_value(&m).unwrap_or_default());

        let company_activity = sqlx::query_as::<_, CompanyActivity>(
            r#"
            INSERT INTO
                company_activities (
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            "#,
        )
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
        .fetch_one(&self.pool)
        .await?;

        Ok(company_activity)
    }
}
