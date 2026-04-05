use thiserror::Error;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("AI error: {0}")]
    AiError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ParseError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::AiError(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::ConfigError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
