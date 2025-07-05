use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use anyhow::{Result, anyhow};
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::database::models::{User, CreateUserRequest, LoginRequest, AuthResponse};
use crate::database::user_repository::UserRepository;
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: String,
    pub exp: usize, // expiration time
}

pub struct AuthService {
    user_repository: UserRepository,
    config: Config,
}

impl AuthService {
    pub fn new(user_repository: UserRepository, config: Config) -> Self {
        Self { user_repository, config }
    }

    pub async fn register(&self, request: CreateUserRequest) -> Result<AuthResponse> {
        // Check if email already exists
        if self.user_repository.email_exists(&request.email).await? {
            return Err(anyhow!("Email already exists"));
        }

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)?;

        // Create user
        let user = User::new(
            request.email,
            password_hash,
            request.name,
            request.role,
        );

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
        let user = self.user_repository
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
        let user = self.user_repository
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
}
