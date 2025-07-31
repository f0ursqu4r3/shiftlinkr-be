use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Returns a 200 OK response with data.
    pub fn success(data: T) -> HttpResponse {
        HttpResponse::Ok().json(Self {
            success: true,
            data: Some(data),
            message: None,
        })
    }

    /// Returns a 201 Created response with data.
    pub fn created(data: T) -> HttpResponse {
        HttpResponse::Created().json(Self {
            success: true,
            data: Some(data),
            message: None,
        })
    }
}

impl ApiResponse<()> {
    /// Returns a 200 OK response with a success message.
    pub fn success_message(message: &str) -> HttpResponse {
        HttpResponse::Ok().json(Self {
            success: true,
            data: None,
            message: Some(message.to_string()),
        })
    }

    /// Builds the body for an error response.
    /// Note: This is used by the custom AppError handler.
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }
    }
}
