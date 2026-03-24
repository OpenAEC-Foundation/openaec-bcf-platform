//! Authentication and authorization.
//!
//! Supports two authentication methods:
//! 1. **Session JWT** — issued after OIDC login, sent as `Authorization: Bearer <jwt>`
//! 2. **API key** — for service-to-service auth, sent as `Authorization: Bearer bcfk_xxx`
//!
//! When `AUTH_ENABLED=false`, all routes are accessible without authentication.

pub mod api_key;
pub mod jwt;
pub mod oidc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::Utc;
use uuid::Uuid;

use crate::db;
use crate::error::AppError;
use crate::state::AppState;

/// Authenticated user identity, extracted from the request.
#[derive(Debug, Clone)]
pub struct AuthUser {
  pub user_id: Uuid,
  pub email: String,
  pub name: String,
}

/// Extractor that requires authentication.
///
/// When `auth_enabled` is false, returns an anonymous placeholder user.
/// When `auth_enabled` is true, validates the `Authorization: Bearer` header
/// as either a session JWT or an API key.
impl FromRequestParts<AppState> for AuthUser {
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    // If auth is disabled, return anonymous user
    if !state.config.auth_enabled {
      return Ok(AuthUser {
        user_id: Uuid::nil(),
        email: "anonymous@local".to_string(),
        name: "Anonymous".to_string(),
      });
    }

    let auth_header = parts
      .headers
      .get("authorization")
      .and_then(|v| v.to_str().ok())
      .ok_or(AppError::Unauthorized)?;

    let token = auth_header
      .strip_prefix("Bearer ")
      .ok_or(AppError::Unauthorized)?;

    // Try API key first (starts with bcfk_)
    if token.starts_with("bcfk_") {
      return validate_api_key_token(token, state).await;
    }

    // Otherwise try session JWT
    validate_jwt_token(token, state).await
  }
}

/// Optional authentication extractor. Always succeeds — returns `None` if not authenticated.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
  type Rejection = std::convert::Infallible;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let user = AuthUser::from_request_parts(parts, state).await.ok();
    Ok(OptionalAuthUser(user))
  }
}

/// Validate a session JWT and look up the user.
async fn validate_jwt_token(token: &str, state: &AppState) -> Result<AuthUser, AppError> {
  let claims = jwt::validate_session_token(token, &state.config.jwt_secret)
    .map_err(|_| AppError::Unauthorized)?;

  // Verify user still exists
  let user = db::users::find_by_id(&state.pool, claims.sub)
    .await
    .map_err(|_| AppError::Unauthorized)?
    .ok_or(AppError::Unauthorized)?;

  Ok(AuthUser {
    user_id: user.id,
    email: user.email,
    name: user.name,
  })
}

/// Validate an API key by prefix lookup + bcrypt verify.
async fn validate_api_key_token(token: &str, state: &AppState) -> Result<AuthUser, AppError> {
  let prefix = api_key::extract_prefix(token).ok_or(AppError::Unauthorized)?;

  let candidates = db::api_keys::find_by_prefix(&state.pool, &prefix)
    .await
    .map_err(|_| AppError::Unauthorized)?;

  for key_row in &candidates {
    // Check expiration
    if let Some(expires) = key_row.expires_at {
      if expires < Utc::now() {
        continue;
      }
    }

    if api_key::verify_api_key(token, &key_row.key_hash) {
      // API keys are service identities — return the key creator as user
      if let Some(creator_id) = key_row.created_by {
        if let Ok(Some(user)) = db::users::find_by_id(&state.pool, creator_id).await {
          return Ok(AuthUser {
            user_id: user.id,
            email: user.email,
            name: user.name,
          });
        }
      }

      // Fallback: return a service account identity
      return Ok(AuthUser {
        user_id: Uuid::nil(),
        email: format!("apikey:{}@service", key_row.prefix),
        name: format!("API Key: {}", key_row.name),
      });
    }
  }

  Err(AppError::Unauthorized)
}
