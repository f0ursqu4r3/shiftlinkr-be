use actix_web::{
    dev::Payload, error::ErrorUnauthorized, web::Data, Error as ActixError, FromRequest,
    HttpRequest,
};
use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

use crate::config::Config;
use crate::database::models::{AuthResponse, CreateUserRequest, LoginRequest, User, UserRole};
use crate::database::repositories::password_reset_repository::PasswordResetTokenRepository;
use crate::database::repositories::user_repository::UserRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: String,
    pub exp: usize, // expiration time
}

impl Claims {
    pub fn is_admin(&self) -> bool {
        self.role.to_lowercase() == "admin"
    }

    pub fn is_manager(&self) -> bool {
        self.role.to_lowercase() == "manager"
    }

    pub fn is_employee(&self) -> bool {
        self.role.to_lowercase() == "employee"
    }

    pub fn user_id(&self) -> &str {
        &self.sub
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
                    if let Some(config) = req.app_data::<Data<Config>>() {
                        match decode::<Claims>(
                            token,
                            &DecodingKey::from_secret(config.jwt_secret.as_ref()),
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
        }

        ready(Err(ErrorUnauthorized(
            "Missing or invalid authorization header",
        )))
    }
}

#[derive(Clone)]
pub struct AuthService {
    user_repository: UserRepository,
    password_reset_repository: PasswordResetTokenRepository,
    config: Config,
}

impl AuthService {
    pub fn new(
        user_repository: UserRepository,
        password_reset_repository: PasswordResetTokenRepository,
        config: Config,
    ) -> Self {
        Self {
            user_repository,
            password_reset_repository,
            config,
        }
    }

    pub async fn register(&self, request: CreateUserRequest) -> Result<AuthResponse> {
        // Check if email already exists
        if self.user_repository.email_exists(&request.email).await? {
            return Err(anyhow!("Email already exists"));
        }

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)?;

        // Create user
        let role = request.role.unwrap_or(UserRole::Employee);
        let user = User::new(request.email, password_hash, request.name, role);

        // Save to database
        self.user_repository.create_user(&user).await?;

        // Generate JWT token
        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse> {
        // Find user by email
        let user = self
            .user_repository
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| anyhow!("Invalid email or password"))?;

        // Verify password
        if !verify(&request.password, &user.password_hash)? {
            return Err(anyhow!("Invalid email or password"));
        }

        // Generate JWT token
        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;

        Ok(token_data.claims)
    }

    pub async fn get_user_from_token(&self, token: &str) -> Result<User> {
        let claims = self.verify_token(token)?;
        let user = self
            .user_repository
            .find_by_id(&claims.sub)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        Ok(user)
    }

    fn generate_token(&self, user: &User) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(self.config.jwt_expiration_days))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            role: user.role.to_string(),
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    /// Request password reset - generates and stores a reset token
    pub async fn forgot_password(&self, email: &str) -> Result<String> {
        // Check if user exists
        let user = self
            .user_repository
            .find_by_email(email)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        // Generate reset token
        let reset_token = self
            .password_reset_repository
            .create_token(&user.id)
            .await?;

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
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<()> {
        // Find and validate the token
        let reset_token = self
            .password_reset_repository
            .find_valid_token(token)
            .await?
            .ok_or_else(|| anyhow!("Invalid or expired reset token"))?;

        // Hash the new password
        let password_hash = hash(new_password, DEFAULT_COST)?;

        // Update user's password
        self.user_repository
            .update_password(&reset_token.user_id, &password_hash)
            .await?;

        // Mark token as used
        self.password_reset_repository
            .mark_token_used(token)
            .await?;

        // Invalidate all other reset tokens for this user
        self.password_reset_repository
            .invalidate_user_tokens(&reset_token.user_id)
            .await?;

        Ok(())
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let user = self
            .user_repository
            .find_by_email(email)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        Ok(user)
    }
}
