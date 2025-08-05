use actix_web::{
    dev::Payload, error::ErrorUnauthorized, Error as ActixError, FromRequest, HttpRequest,
};
use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::config::config;
use crate::database::models::{AuthResponse, CompanyRole, CreateUserInput, LoginInput, User};
use crate::database::repositories::{
    company as company_repo, password_reset as password_reset_repo, user as user_repo,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub company_id: Option<Uuid>, // optional company ID for company-specific roles
    pub role: Option<CompanyRole>, // user role (admin, manager, employee)
    pub exp: usize,               // expiration time
}

impl Claims {
    pub fn user_id(&self) -> Uuid {
        self.sub
    }
    pub fn is_admin(&self) -> bool {
        self.role == Some(CompanyRole::Admin)
    }
    pub fn is_manager(&self) -> bool {
        self.role == Some(CompanyRole::Manager)
    }
    pub fn is_employee(&self) -> bool {
        self.role == Some(CompanyRole::Employee)
    }
    pub fn is_manager_or_admin(&self) -> bool {
        self.is_manager() || self.is_admin()
    }
}

impl FromRequest for Claims {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..]; // Remove "Bearer " prefix

                    // Get the config from app data
                    match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(config().jwt_secret.as_ref()),
                        &Validation::new(Algorithm::HS256),
                    ) {
                        Ok(token_data) => {
                            return ready(Ok(token_data.claims));
                        }
                        Err(_) => {
                            return ready(Err(ErrorUnauthorized("Invalid token")));
                        }
                    }
                }
            }
        }

        ready(Err(ErrorUnauthorized(
            "Missing or invalid authorization header",
        )))
    }
}

pub async fn register(request: CreateUserInput) -> Result<AuthResponse> {
    // Check if email already exists
    if user_repo::email_exists(&request.email).await? {
        return Err(anyhow!("Email already exists"));
    }

    // Hash password
    let password_hash = hash(&request.password, DEFAULT_COST)?;

    // Create user
    let user = User::new(request.email, password_hash, request.name);

    // Save to database
    user_repo::create_user(&user).await?;

    // Generate JWT token - for now we'll need to handle this differently since role is company-specific
    // TODO: Update this to handle company-specific roles
    let token = generate_token(&user, None, None)?;

    Ok(AuthResponse {
        token,
        user: user.into(),
        company: None, // No company info on registration
    })
}

pub async fn login(request: LoginInput) -> Result<AuthResponse> {
    // Find user by email
    let user = user_repo::find_by_email(&request.email)
        .await?
        .ok_or_else(|| anyhow!("Invalid email or password"))?;

    // Verify password
    if !verify(&request.password, &user.password_hash)? {
        return Err(anyhow!("Invalid email or password"));
    }

    // Get user's companies
    let companies = company_repo::get_companies_for_user(user.id).await?;

    let primary_company = companies
        .iter()
        .find(|c| c.is_primary)
        .or_else(|| companies.first())
        .cloned();

    let company_id = primary_company.as_ref().map(|c| c.id);

    let role = match primary_company {
        Some(ref company) => Some(company.role.clone()),
        None => None,
    };

    // Generate JWT token
    let token = generate_token(&user, company_id, role)?;

    Ok(AuthResponse {
        token,
        user: user.into(),
        company: primary_company,
    })
}

pub fn verify_token(token: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config().jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}

pub async fn get_user_from_token(token: &str) -> Result<User> {
    let claims = verify_token(token)?;
    let user_id = claims.sub;
    let user = user_repo::find_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

    Ok(user)
}

pub async fn generate_company_token(user_id: Uuid, company_id: Uuid) -> Result<String> {
    let user = user_repo::find_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

    let company = company_repo::find_by_id(company_id)
        .await?
        .ok_or_else(|| anyhow!("Company not found"))?;
    // Get user's role in the company (you'll need to implement this in CompanyRepository)
    // let role = company_repository.get_user_role_in_company(user_id, company_id).await?;
    let role = Some(CompanyRole::Employee); // Placeholder
    let retreived_company_id = Some(company.id);

    let token = generate_token(&user, retreived_company_id, role)?;
    Ok(token)
}

// Updated generate_token method
fn generate_token(
    user: &User,
    company_id: Option<Uuid>,
    role: Option<CompanyRole>,
) -> Result<String> {
    // Get the config
    let config = config();
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(config.jwt_expiration_days))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        company_id,
        role,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )?;

    Ok(token)
}

/// Request password reset - generates and stores a reset token
pub async fn forgot_password(email: &str) -> Result<String> {
    // Check if user exists
    let user = user_repo::find_by_email(email)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

    // Generate reset token
    let reset_token = password_reset_repo::create_token(user.id).await?;

    // In a real application, you would send this token via email
    // For development, we'll return it directly
    println!(
        "🔗 Password reset token for {}: {}",
        email, reset_token.token
    );
    println!(
        "🌐 Reset link: http://localhost:3000/auth/reset-password?token={}",
        reset_token.token
    );

    Ok(reset_token.token)
}

/// Reset password using a valid token
pub async fn reset_password(token: &str, new_password: &str) -> Result<()> {
    // Find and validate the token
    let reset_token = password_reset_repo::find_valid_token(token)
        .await?
        .ok_or_else(|| anyhow!("Invalid or expired reset token"))?;

    // Hash the new password
    let password_hash = hash(new_password, DEFAULT_COST)?;

    // Update user's password
    user_repo::update_password(reset_token.user_id, &password_hash).await?;

    // Mark token as used
    password_reset_repo::mark_token_used(token).await?;

    // Invalidate all other reset tokens for this user
    password_reset_repo::invalidate_user_tokens(reset_token.user_id).await?;

    Ok(())
}

/// Get user by email
pub async fn get_user_by_email(email: &str) -> Result<User> {
    let user = user_repo::find_by_email(email)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

    Ok(user)
}

pub async fn switch_company(user_id: Uuid, new_company_id: Uuid) -> Result<AuthResponse> {
    // Check if user belongs to the new company
    match company_repo::check_user_company_access(user_id, new_company_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(anyhow!("User does not belong to the new company")),
        Err(e) => return Err(anyhow!("Error checking user company access: {}", e)),
    }

    let user = user_repo::find_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

    let company = company_repo::find_user_company_info_by_id(user_id, new_company_id)
        .await?
        .ok_or_else(|| anyhow!("Company not found"))?;

    let role = Some(company.role.clone());

    let token = generate_token(&user, Some(new_company_id), role)?;

    Ok(AuthResponse {
        token,
        user: user.into(),
        company: Some(company),
    })
}
