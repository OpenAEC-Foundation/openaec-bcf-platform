//! Event (audit log) route handlers (BCF v2.1).
//!
//! GET /bcf/2.1/projects/{project_id}/topics/{topic_id}/events
//! GET /bcf/2.1/projects/{project_id}/events

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::event::EventResponse;
use crate::state::AppState;

/// Topic event routes (nested under topics/{topic_id}/events).
pub fn topic_event_routes() -> Router<AppState> {
  Router::new().route("/", get(list_topic_events))
}

/// Project event routes (nested under projects/{project_id}/events).
pub fn project_event_routes() -> Router<AppState> {
  Router::new().route("/", get(list_project_events))
}

/// GET /topics/{topic_id}/events — List events for a topic.
async fn list_topic_events(
  State(state): State<AppState>,
  Path((_project_id, topic_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<EventResponse>>> {
  let rows = db::events::list_topic_events(&state.pool, topic_id).await?;
  let events: Vec<EventResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(events))
}

/// GET /projects/{project_id}/events — List events for all topics in a project.
async fn list_project_events(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<Vec<EventResponse>>> {
  // Verify project exists
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  let rows = db::events::list_project_events(&state.pool, project_id).await?;
  let events: Vec<EventResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(events))
}
