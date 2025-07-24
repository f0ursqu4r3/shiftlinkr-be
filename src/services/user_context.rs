use actix_web::{
    dev::Payload, error::ErrorUnauthorized, web::Data, Error as ActixError, FromRequest,
    HttpRequest,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::database::models::{
    company::{CompanyInfo, CompanyRole},
    user::User,
};
use crate::database::repositories::{company::CompanyRepository, user::UserRepository};
use crate::services::auth::Claims;

/// User context that contains the current user and their company information
/// This is automatically injected into requests via the FromRequest trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user: User,
    pub company: Option<CompanyInfo>,
}

impl UserContext {
    /// Create a new UserContext from claims and repositories
    pub async fn from_claims(
        claims: &Claims,
        user_repo: &UserRepository,
        company_repo: &CompanyRepository,
    ) -> Result<Self> {
        // Get the user
        let user = user_repo
            .find_by_id(claims.sub)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        // Get the company if company_id is present in claims
        let company = if let Some(company_id) = claims.company_id {
            company_repo
                .find_company_info_by_id(user.id, company_id)
                .await?
        } else {
            None
        };

        Ok(UserContext { user, company })
    }

    /// Get the user ID
    pub fn user_id(&self) -> Uuid {
        self.user.id
    }

    /// Get the user email
    pub fn user_email(&self) -> &str {
        &self.user.email
    }

    /// Get the company ID if available
    pub fn company_id(&self) -> Option<Uuid> {
        match self.company {
            Some(ref company) => Some(company.id),
            None => None,
        }
    }

    /// Get the user's role in the current company
    pub fn role(&self) -> Option<&CompanyRole> {
        match self.company {
            Some(ref company) => Some(&company.role),
            None => None,
        }
    }

    /// Check if user is admin in current company
    pub fn is_admin(&self) -> bool {
        matches!(self.role(), Some(CompanyRole::Admin))
    }

    /// Check if user is manager in current company
    pub fn is_manager(&self) -> bool {
        matches!(self.role(), Some(CompanyRole::Manager))
    }

    /// Check if user is employee in current company
    pub fn is_employee(&self) -> bool {
        matches!(self.role(), Some(CompanyRole::Employee))
    }

    /// Check if user is manager or admin
    pub fn is_manager_or_admin(&self) -> bool {
        self.is_manager() || self.is_admin()
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &CompanyRole) -> bool {
        self.role() == Some(role)
    }

    /// Check if user belongs to a specific company
    pub fn belongs_to_company(&self, company_id: Uuid) -> bool {
        self.company_id() == Some(company_id)
    }

    /// Get company name if available
    pub fn company_name(&self) -> Option<&str> {
        self.company.as_ref().map(|c| c.name.as_str())
    }

    /// Check if user can access resource owned by another user
    /// Admins and managers can access any resource in their company
    /// Employees can only access their own resources
    pub fn can_access_user_resource(&self, resource_owner_id: Uuid) -> bool {
        // User can always access their own resources
        if self.user_id() == resource_owner_id {
            return true;
        }

        // Admins and managers can access resources of users in their company
        self.is_manager_or_admin()
    }

    /// Check if user can manage another user (for admin operations)
    /// Only admins can manage other users
    pub fn can_manage_user(&self, target_user_id: Uuid) -> bool {
        // Users cannot manage themselves through admin operations
        if self.user_id() == target_user_id {
            return false;
        }

        // Only admins can manage other users
        self.is_admin()
    }
}

impl FromRequest for UserContext {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // First extract claims
        let claims_future = Claims::from_request(req, payload);

        match claims_future.into_inner() {
            Ok(_claims) => {
                // Get repositories from app data
                let user_repo = req.app_data::<Data<UserRepository>>();
                let company_repo = req.app_data::<Data<CompanyRepository>>();

                if let (Some(_user_repo), Some(_company_repo)) = (user_repo, company_repo) {
                    // Since we can't make async calls here, we'll need to handle this differently
                    // We'll return an error and suggest using the service directly in handlers
                    ready(Err(ErrorUnauthorized(
                        "UserContext cannot be extracted directly. Use UserContextService in your handler instead.",
                    )))
                } else {
                    ready(Err(ErrorUnauthorized(
                        "Required repositories not found in app data",
                    )))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}

/// Service for creating UserContext from requests
/// Use this in your handlers instead of trying to extract UserContext directly
#[derive(Clone)]
pub struct UserContextService {
    user_repository: UserRepository,
    company_repository: CompanyRepository,
}

impl UserContextService {
    pub fn new(user_repository: UserRepository, company_repository: CompanyRepository) -> Self {
        Self {
            user_repository,
            company_repository,
        }
    }

    /// Extract UserContext from a request
    pub async fn extract_context(&self, req: &HttpRequest) -> Result<UserContext> {
        // Extract claims from the request
        let mut payload = Payload::None;
        let claims_result = Claims::from_request(req, &mut payload)
            .into_inner()
            .map_err(|e| anyhow!(e.to_string()))?;

        // Create UserContext from claims
        UserContext::from_claims(
            &claims_result,
            &self.user_repository,
            &self.company_repository,
        )
        .await
    }

    /// Create UserContext from existing claims (useful for testing or when you already have claims)
    pub async fn from_claims(&self, claims: &Claims) -> Result<UserContext> {
        UserContext::from_claims(claims, &self.user_repository, &self.company_repository).await
    }

    /// Get UserContext for a specific user (admin operation)
    pub async fn get_user_context(
        &self,
        user_id: Uuid,
        company_id: Option<Uuid>,
    ) -> Result<UserContext> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        let company = if let Some(company_id) = company_id {
            self.company_repository
                .find_company_info_by_id(user.id, company_id)
                .await?
        } else {
            None
        };

        Ok(UserContext { user, company })
    }
}

/// Helper macro for extracting user context in handlers
/// Usage: let user_context = extract_user_context!(user_context_service, req)?;
#[macro_export]
macro_rules! extract_user_context {
    ($service:expr, $req:expr) => {
        $service.extract_context($req).await.map_err(|e| {
            actix_web::error::ErrorUnauthorized(format!("Failed to extract user context: {}", e))
        })?
    };
}

/// Helper function for handlers that need user context
/// This is an alternative to the macro
pub async fn get_user_context(
    service: &UserContextService,
    req: &HttpRequest,
) -> Result<UserContext, ActixError> {
    service
        .extract_context(req)
        .await
        .map_err(|e| ErrorUnauthorized(format!("Failed to extract user context: {}", e)))
}
