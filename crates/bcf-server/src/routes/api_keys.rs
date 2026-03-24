//! API key management route handlers.
//!
//! POST   /api/v1/projects/{project_id}/api-keys       — Create API key
//! GET    /api/v1/projects/{project_id}/api-keys        — List API keys
//! DELETE /api/v1/projects/{project_id}/api-keys/{id}   — Revoke API key

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::api_key::generate_api_key;
use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::api_key::{ApiKeyCreatedResponse, ApiKeyResponse, CreateApiKeyRequest};
use crate::state::AppState;

/// API key routes (nested under /api/v1/projects).
pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/{project_id}/api-keys", get(list_keys).post(create_key))
    .route("/{project_id}/api-keys/{key_id}", delete(delete_key))
}

/// POST /api-keys — Create a new API key for a project.
async fn create_key(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
  auth: AuthUser,
  Json(body): Json<CreateApiKeyRequest>,
) -> AppResult<(StatusCode, Json<ApiKeyCreatedResponse>)> {
  if body.name.trim().is_empty() {
    return Err(AppError::BadRequest("name is required".to_string()));
  }

  // Verify project exists
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  // Generate the key
  let generated = generate_api_key()
    .map_err(|e| AppError::Internal(format!("key generation failed: {e}")))?;

  // Store in database
  let created_by = if auth.user_id.is_nil() {
    None
  } else {
    Some(auth.user_id)
  };

  let row = db::api_keys::create_api_key(
    &state.pool,
    project_id,
    &body.name,
    &generated.key_hash,
    &generated.prefix,
    created_by,
    body.expires_at,
  )
  .await?;

  tracing::info!(
    key_id = %row.id,
    project_id = %project_id,
    prefix = %generated.prefix,
    "API key created"
  );

  Ok((
    StatusCode::CREATED,
    Json(ApiKeyCreatedResponse {
      id: row.id,
      name: row.name,
      prefix: generated.prefix,
      key: generated.raw_key,
      expires_at: row.expires_at,
    }),
  ))
}

/// GET /api-keys — List all API keys for a project.
async fn list_keys(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
  _auth: AuthUser,
) -> AppResult<Json<Vec<ApiKeyResponse>>> {
  let rows = db::api_keys::list_api_keys(&state.pool, project_id).await?;
  let keys: Vec<ApiKeyResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(keys))
}

/// DELETE /api-keys/{key_id} — Revoke an API key.
async fn delete_key(
  State(state): State<AppState>,
  Path((project_id, key_id)): Path<(Uuid, Uuid)>,
  _auth: AuthUser,
) -> AppResult<StatusCode> {
  let deleted = db::api_keys::delete_api_key(&state.pool, project_id, key_id).await?;
  if deleted {
    tracing::info!(key_id = %key_id, project_id = %project_id, "API key revoked");
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(format!("API key {key_id}")))
  }
}
