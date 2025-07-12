use crate::database::models::CreateActivityRequest;
use sqlx::{Result, SqlitePool};

#[derive(Clone)]
pub struct ActivityRepository {
    pool: SqlitePool,
}

impl ActivityRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Log a new activity  
    pub async fn log_activity(&self, request: CreateActivityRequest) -> Result<i64> {
        let metadata_json = request
            .metadata
            .map(|m| serde_json::to_string(&m).unwrap_or_default());

        let result = sqlx::query!(
            r#"
            INSERT INTO company_activities 
            (company_id, user_id, activity_type, entity_type, entity_id, action, description, metadata, ip_address, user_agent)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            request.company_id,
            request.user_id,
            request.activity_type,
            request.entity_type,
            request.entity_id,
            request.action,
            request.description,
            metadata_json,
            request.ip_address,
            request.user_agent
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }
}
