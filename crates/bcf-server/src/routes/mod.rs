//! API route definitions and handler modules.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::state::AppState;

pub mod api_keys;
pub mod auth_routes;
pub mod bcfio;
pub mod cloud;
pub mod comments;
pub mod events;
pub mod extensions;
pub mod health;
pub mod members;
pub mod projects;
pub mod topics;
pub mod users;
pub mod viewpoints;

/// Build the complete API router with all BCF v2.1 and platform routes.
pub fn api_router() -> Router<AppState> {
  Router::new()
    .merge(health::routes())
    .merge(auth_routes::routes())
    .merge(users::local_auth_routes())
    .route("/bcf/versions", get(bcf_versions))
    .nest("/bcf/2.1", bcf_routes())
    .nest("/api/v1", platform_routes())
    .nest("/api", cloud::routes())
}

/// BCF v2.1 standard-compliant routes.
fn bcf_routes() -> Router<AppState> {
  Router::new()
    .route("/auth", get(bcf_auth))
    .nest("/projects", projects::bcf_project_routes())
}

/// Platform-specific routes (non-BCF standard).
fn platform_routes() -> Router<AppState> {
  Router::new()
    .nest("/projects", projects::platform_project_routes())
    .nest("/projects", bcfio::routes())
    .nest("/projects", api_keys::routes())
    .nest("/projects", members::routes())
    .merge(users::routes())
}

// -- BCF root-level handlers --------------------------------------------------

/// BCF version info response.
#[derive(Debug, Serialize)]
struct BcfVersion {
  version_id: &'static str,
  detailed_version: &'static str,
}

#[derive(Debug, Serialize)]
struct BcfVersionsResponse {
  versions: Vec<BcfVersion>,
}

/// GET /bcf/versions — BCF API version discovery.
async fn bcf_versions() -> Json<BcfVersionsResponse> {
  Json(BcfVersionsResponse {
    versions: vec![BcfVersion {
      version_id: "2.1",
      detailed_version: "https://bcf.open-aec.com/bcf/2.1",
    }],
  })
}

/// BCF auth discovery response.
#[derive(Debug, Serialize)]
struct BcfAuthResponse {
  oauth2_auth_url: String,
  oauth2_token_url: String,
  supported_oauth2_flows: Vec<&'static str>,
}

/// GET /bcf/2.1/auth — OAuth2 endpoint discovery for BCF clients.
async fn bcf_auth(
  State(state): State<AppState>,
) -> Result<Json<BcfAuthResponse>, crate::error::AppError> {
  let oidc = state
    .oidc_client
    .as_ref()
    .ok_or_else(|| crate::error::AppError::NotFound("auth not configured".to_string()))?;

  Ok(Json(BcfAuthResponse {
    oauth2_auth_url: oidc.authorization_endpoint().to_string(),
    oauth2_token_url: oidc.token_endpoint().to_string(),
    supported_oauth2_flows: vec!["authorization_code_grant"],
  }))
}
