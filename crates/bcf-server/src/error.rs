//! Centralized error handling with automatic HTTP response mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Application-level errors that map to HTTP responses.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
  #[error("not found: {0}")]
  NotFound(String),

  #[error("bad request: {0}")]
  BadRequest(String),

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("unauthorized")]
  Unauthorized,

  #[error("forbidden")]
  Forbidden,

  #[error("database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("internal error: {0}")]
  Internal(String),
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, message) = match &self {
      AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
      AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
      AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
      AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
      AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
      AppError::Database(err) => {
        tracing::error!("database error: {err}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "internal server error".to_string(),
        )
      }
      AppError::Internal(msg) => {
        tracing::error!("internal error: {msg}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "internal server error".to_string(),
        )
      }
    };

    let body = json!({ "error": message });
    (status, axum::Json(body)).into_response()
  }
}

/// Shorthand result type for handlers.
pub type AppResult<T> = Result<T, AppError>;
