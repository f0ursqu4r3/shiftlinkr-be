use crate::database::models::{ActivityType, CreateActivityRequest, EntityType};
use crate::database::repositories::ActivityRepository;
use actix_web::HttpRequest;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ActivityLogger {
    repository: ActivityRepository,
}

impl ActivityLogger {
    pub fn new(repository: ActivityRepository) -> Self {
        Self { repository }
    }

    /// Extract client info from HTTP request
    fn extract_client_info(&self, req: &HttpRequest) -> (Option<String>, Option<String>) {
        let ip_address = req.connection_info().peer_addr().map(|addr| {
            // Remove port if present
            if addr.contains(':') {
                addr.split(':').next().unwrap_or(addr).to_string()
            } else {
                addr.to_string()
            }
        });

        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        (ip_address, user_agent)
    }

    /// Log user management activity
    pub async fn log_user_activity(
        &self,
        company_id: i64,
        user_id: Option<i64>,
        target_user_id: i64,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityRequest {
            company_id,
            user_id,
            activity_type: ActivityType::USER_MANAGEMENT.to_string(),
            entity_type: EntityType::USER.to_string(),
            entity_id: target_user_id,
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    /// Log authentication activity
    pub async fn log_auth_activity(
        &self,
        company_id: i64,
        user_id: Option<i64>,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityRequest {
            company_id,
            user_id,
            activity_type: ActivityType::AUTHENTICATION.to_string(),
            entity_type: EntityType::USER.to_string(),
            entity_id: user_id.unwrap_or(0), // Use 0 for failed logins where user_id is unknown
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    /// Generic activity logging for custom cases
    pub async fn log_activity(
        &self,
        company_id: i64,
        user_id: Option<i64>,
        activity_type: String,
        entity_type: String,
        entity_id: i64,
        action: String,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityRequest {
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
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }
}
