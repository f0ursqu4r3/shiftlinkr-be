use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    // Success with data and message
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    // Success with message
    pub fn success_with_message(data: Option<T>, message: &str) -> Self {
        Self {
            success: true,
            data,
            message: Some(message.to_string()),
        }
    }

    // Error with data (e.g., validation errors)
    pub fn error_with_data(data: T, message: &str) -> Self {
        Self {
            success: false,
            data: Some(data),
            message: Some(message.to_string()),
        }
    }

    // Error with data, no message
    pub fn error_data_only(data: T) -> Self {
        Self {
            success: false,
            data: Some(data),
            message: None,
        }
    }
}

impl ApiResponse<()> {
    // Error response (no data)
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }
    }
}
