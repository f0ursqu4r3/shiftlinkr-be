use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

use crate::handlers::shared::ApiResponse;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Internal server error{}", .0.as_ref().map_or("".to_string(), |s| format!(": {}", s)))]
    InternalServerError(Option<String>),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_message = self.to_string();

        log::error!(
            "Request failed with status {}: {}",
            status_code,
            error_message
        );

        let response_body = ApiResponse::<()>::error(&error_message);

        HttpResponse::build(status_code).json(response_body)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        log::error!("Database error: {}", error);
        AppError::DatabaseError(error)
    }
}

impl AppError {
    pub fn internal_server_error_message(message: impl Into<String>) -> Self {
        AppError::InternalServerError(Some(message.into()))
    }

    pub fn internal_server_error() -> Self {
        AppError::InternalServerError(None)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        log::error!("Anyhow error: {}", error);

        // Check if this is a sqlx::Error and handle it appropriately
        if error.is::<sqlx::Error>() {
            // Downcast the error to sqlx::Error by consuming the anyhow::Error
            match error.downcast::<sqlx::Error>() {
                Ok(sqlx_err) => return AppError::DatabaseError(sqlx_err),
                Err(original_error) => {
                    // If downcast fails somehow, fall back to the original error
                    return AppError::InternalServerError(Some(original_error.to_string()));
                }
            }
        }

        AppError::InternalServerError(Some(error.to_string()))
    }
}
