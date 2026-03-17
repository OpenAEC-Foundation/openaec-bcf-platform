//! API route definitions and handler modules.

use axum::Router;

use crate::state::AppState;

pub mod comments;
pub mod health;
pub mod projects;
pub mod topics;

/// Build the complete API router with all BCF v2.1 and platform routes.
pub fn api_router() -> Router<AppState> {
  Router::new()
    .merge(health::routes())
    .nest("/bcf/2.1", bcf_routes())
    .nest("/api/v1", platform_routes())
}

/// BCF v2.1 standard-compliant routes.
fn bcf_routes() -> Router<AppState> {
  Router::new()
    .nest("/projects", projects::bcf_project_routes())
}

/// Platform-specific routes (non-BCF standard).
fn platform_routes() -> Router<AppState> {
  Router::new()
    .nest("/projects", projects::platform_project_routes())
}
