use crate::database::models::{ActivityType, CreateActivityInput, EntityType};
use crate::database::repositories::ActivityRepository;
use actix_web::HttpRequest;
use std::collections::HashMap;
use uuid::Uuid;

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

    /// Generic activity logging for custom cases
    pub async fn log_activity(
        &self,
        company_id: Uuid,
        user_id: Option<Uuid>,
        activity_type: String,
        entity_type: String,
        entity_id: Uuid,
        action: String,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
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

    /// Log user management activity
    pub async fn log_user_activity(
        &self,
        company_id: Uuid,
        user_id: Option<Uuid>,
        target_user_id: Uuid,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
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
        company_id: Uuid,
        user_id: Option<Uuid>,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
            company_id,
            user_id,
            activity_type: ActivityType::AUTHENTICATION.to_string(),
            entity_type: EntityType::USER.to_string(),
            entity_id: user_id.unwrap_or(Uuid::nil()), // Use nil UUID for failed logins where user_id is unknown
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    /// Log location management activity
    pub async fn log_location_activity(
        &self,
        company_id: Uuid,
        user_id: Option<Uuid>,
        location_id: Uuid,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
            company_id,
            user_id,
            activity_type: ActivityType::LOCATION_MANAGEMENT.to_string(),
            entity_type: EntityType::LOCATION.to_string(),
            entity_id: location_id,
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    /// Log team management activity
    pub async fn log_team_activity(
        &self,
        company_id: Uuid,
        user_id: Option<Uuid>,
        team_id: Uuid,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
            company_id,
            user_id,
            activity_type: ActivityType::TEAM_MANAGEMENT.to_string(),
            entity_type: EntityType::TEAM.to_string(),
            entity_id: team_id,
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    /// Log shift management activity
    pub async fn log_shift_activity(
        &self,
        company_id: Uuid,
        user_id: Option<Uuid>,
        shift_id: Uuid,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
            company_id,
            user_id,
            activity_type: ActivityType::SHIFT_MANAGEMENT.to_string(),
            entity_type: EntityType::SHIFT.to_string(),
            entity_id: shift_id,
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    /// Log time off management activity
    pub async fn log_time_off_activity(
        &self,
        company_id: Uuid,
        user_id: Option<Uuid>,
        time_off_id: Uuid,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
            company_id,
            user_id,
            activity_type: ActivityType::TIME_OFF_MANAGEMENT.to_string(),
            entity_type: EntityType::TIME_OFF.to_string(),
            entity_id: time_off_id,
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    /// Log shift swap activity
    pub async fn log_shift_swap_activity(
        &self,
        company_id: Uuid,
        user_id: Option<Uuid>,
        swap_id: Uuid,
        action: &str,
        description: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
        req: &HttpRequest,
    ) -> Result<(), sqlx::Error> {
        let (ip_address, user_agent) = self.extract_client_info(req);

        let request = CreateActivityInput {
            company_id,
            user_id,
            activity_type: ActivityType::SHIFT_MANAGEMENT.to_string(),
            entity_type: EntityType::SHIFT_SWAP.to_string(),
            entity_id: swap_id,
            action: action.to_string(),
            description,
            metadata,
            ip_address,
            user_agent,
        };

        self.repository.log_activity(request).await?;
        Ok(())
    }

    pub fn metadata(pairs: Vec<(&str, String)>) -> HashMap<String, serde_json::Value> {
        pairs
            .into_iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v)))
            .collect()
    }
}
