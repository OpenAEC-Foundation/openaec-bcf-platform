//! Project extensions route handler (BCF v2.1).
//!
//! GET /bcf/2.1/projects/{project_id}/extensions

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::extension::ExtensionResponse;
use crate::state::AppState;

/// Extension routes (nested under projects/{project_id}/extensions).
pub fn routes() -> Router<AppState> {
  Router::new().route("/", get(get_extensions))
}

/// GET /extensions — Get project extensions (auto-creates defaults if missing).
async fn get_extensions(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<ExtensionResponse>> {
  // Verify project exists
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  let row = db::extensions::get_or_create_extensions(&state.pool, project_id).await?;
  Ok(Json(row.into()))
}
