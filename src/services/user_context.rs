use actix_web::{dev::Payload, FromRequest, HttpRequest};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::database::repositories::{company as company_repo, user as user_repo};
use crate::services::auth::Claims;
use crate::user_context;
use crate::{
    database::models::{
        company::{CompanyInfo, CompanyRole},
        user::User,
    },
    error::AppError,
};

/// User context that contains the current user and their company information
/// This is created per-request and contains the authenticated user's information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user: User,
    pub company: Option<CompanyInfo>,
}

impl UserContext {
    /// Create a new UserContext from claims and repositories
    pub async fn from_claims(claims: &Claims) -> Result<Self> {
        // Get the user
        let user = user_repo::find_by_id(claims.sub)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        // Get the company if company_id is present in claims
        let company = if let Some(company_id) = claims.company_id {
            company_repo::find_user_company_info_by_id(user.id, company_id).await?
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
        self.company.as_ref().map(|c| c.id)
    }

    /// Get the user's role in the current company
    pub fn role(&self) -> Option<&CompanyRole> {
        self.company.as_ref().map(|c| &c.role)
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

    pub fn strict_company_id(&self) -> Result<Uuid, AppError> {
        self.company_id().ok_or_else(|| {
            AppError::PermissionDenied("User does not belong to a company".to_string())
        })
    }

    pub fn requires_admin(&self) -> Result<(), AppError> {
        if !self.is_admin() {
            return Err(AppError::PermissionDenied(
                "Admin access required".to_string(),
            ));
        }
        Ok(())
    }

    pub fn requires_admin_or(&self, message: Option<String>) -> Result<(), AppError> {
        if !self.is_manager_or_admin() {
            return Err(AppError::PermissionDenied(
                message.unwrap_or_else(|| "Admin or manager access required".to_string()),
            ));
        }
        Ok(())
    }

    pub fn requires_manager(&self) -> Result<(), AppError> {
        if !self.is_manager_or_admin() {
            return Err(AppError::PermissionDenied(
                "Manager access required".to_string(),
            ));
        }
        Ok(())
    }

    pub fn requires_manager_or(&self, message: &str) -> Result<(), AppError> {
        if !self.is_manager_or_admin() {
            return Err(AppError::PermissionDenied(message.to_string()));
        }
        Ok(())
    }

    pub fn requires_same_user(&self, target_user_id: Uuid) -> Result<(), AppError> {
        if !self.is_manager_or_admin() || self.user_id() != target_user_id {
            return Err(AppError::PermissionDenied(
                "Access denied: you can only access your own resources".to_string(),
            ));
        }
        Ok(())
    }

    pub fn requires_same_user_or(
        &self,
        target_user_id: Uuid,
        message: &str,
    ) -> Result<(), AppError> {
        if !self.is_manager_or_admin() || self.user_id() != target_user_id {
            return Err(AppError::PermissionDenied(message.to_string()));
        }
        Ok(())
    }

    pub fn requires_same_company(&self, target_company_id: Uuid) -> Result<(), AppError> {
        if self.company_id() != Some(target_company_id) {
            return Err(AppError::PermissionDenied(
                "Access denied: you can only access resources in your own company".to_string(),
            ));
        }
        Ok(())
    }
    pub fn requires_same_company_or(
        &self,
        target_company_id: Uuid,
        message: &str,
    ) -> Result<(), AppError> {
        if self.company_id() != Some(target_company_id) {
            return Err(AppError::PermissionDenied(message.to_string()));
        }
        Ok(())
    }
}

/// Extract UserContext from a request
pub async fn extract_context(req: &HttpRequest) -> Result<UserContext, AppError> {
    // Extract claims from the request
    let mut payload = Payload::None;
    let claims_result = Claims::from_request(req, &mut payload);

    // Since FromRequest returns a Ready<Result<Claims, ActixError>>, we need to get the inner result
    let claims = claims_result
        .into_inner()
        .map_err(|_| AppError::Unauthorized)?;

    let user_context = user_context::UserContext::from_claims(&claims)
        .await
        .map_err(|_| AppError::Unauthorized)?;
    // Create UserContext from claims
    Ok(user_context)
}

/// Create UserContext from existing claims (useful for testing or when you already have claims)
pub async fn from_claims(claims: &Claims) -> Result<UserContext, AppError> {
    user_context::UserContext::from_claims(claims)
        .await
        .map_err(|_| AppError::Unauthorized)
}

/// Get UserContext for a specific user (admin operation)
pub async fn get_user_context(
    user_id: Uuid,
    company_id: Option<Uuid>,
) -> Result<UserContext, AppError> {
    let user = user_repo::find_by_id(user_id)
        .await
        .map_err(|e| AppError::DatabaseError(e))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let company = if let Some(company_id) = company_id {
        company_repo::find_user_company_info_by_id(user.id, company_id).await?
    } else {
        None
    };

    Ok(UserContext { user, company })
}
