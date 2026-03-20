//! Viewpoint CRUD route handlers with snapshot storage.
//!
//! Nested under /bcf/2.1/projects/{project_id}/topics/{topic_id}/viewpoints

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use base64::Engine;
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::viewpoint::{
  CreateViewpointRequest, UpdateViewpointRequest, ViewpointResponse,
};
use crate::state::AppState;

/// Viewpoint routes (nested under topics/{topic_id}/viewpoints).
pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/", get(list_viewpoints).post(create_viewpoint))
    .route(
      "/{viewpoint_id}",
      get(get_viewpoint).put(update_viewpoint).delete(delete_viewpoint),
    )
    .route("/{viewpoint_id}/snapshot", get(get_snapshot))
}

/// GET /viewpoints — List all viewpoints for a topic.
async fn list_viewpoints(
  State(state): State<AppState>,
  Path((_project_id, topic_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<ViewpointResponse>>> {
  let rows = db::viewpoints::list_viewpoints(&state.pool, topic_id).await?;
  let viewpoints: Vec<ViewpointResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(viewpoints))
}

/// GET /viewpoints/{viewpoint_id} — Get a single viewpoint.
async fn get_viewpoint(
  State(state): State<AppState>,
  Path((_project_id, _topic_id, viewpoint_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<ViewpointResponse>> {
  let row = db::viewpoints::get_viewpoint(&state.pool, viewpoint_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("viewpoint {viewpoint_id}")))?;
  Ok(Json(row.into()))
}

/// POST /viewpoints — Create a new viewpoint with optional snapshot.
async fn create_viewpoint(
  State(state): State<AppState>,
  Path((_project_id, topic_id)): Path<(Uuid, Uuid)>,
  Json(body): Json<CreateViewpointRequest>,
) -> AppResult<(StatusCode, Json<ViewpointResponse>)> {
  // Verify topic exists
  db::topics::get_topic_by_id(&state.pool, topic_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("topic {topic_id}")))?;

  // Decode and store snapshot if provided
  let snapshot_path = if let Some(ref b64) = body.snapshot_data {
    let data = base64::engine::general_purpose::STANDARD
      .decode(b64)
      .map_err(|e| AppError::BadRequest(format!("invalid base64 snapshot: {e}")))?;

    let temp_id = Uuid::new_v4();
    let path = state
      .storage
      .save(topic_id, temp_id, &data)
      .await
      .map_err(|e| AppError::Internal(format!("snapshot save failed: {e}")))?;
    Some(path)
  } else {
    None
  };

  let row = db::viewpoints::create_viewpoint(
    &state.pool,
    topic_id,
    snapshot_path.as_deref(),
    body.camera.as_ref(),
    body.components.as_ref(),
  )
  .await?;

  // If we saved snapshot with a temp ID, rename to actual viewpoint ID
  if let (Some(old_path), true) = (&snapshot_path, snapshot_path.is_some()) {
    let new_path = format!("{topic_id}/{}.png", row.id);
    if *old_path != new_path {
      // Load, save with correct name, delete old
      if let Ok(data) = state.storage.load(old_path).await {
        let _ = state.storage.save(topic_id, row.id, &data).await;
        let _ = state.storage.delete(old_path).await;
        // Update the path in the database
        let _ = db::viewpoints::update_viewpoint(
          &state.pool,
          row.id,
          Some(&new_path),
          None,
          None,
        )
        .await;
      }
    }
  }

  Ok((StatusCode::CREATED, Json(row.into())))
}

/// PUT /viewpoints/{viewpoint_id} — Update an existing viewpoint.
async fn update_viewpoint(
  State(state): State<AppState>,
  Path((_project_id, topic_id, viewpoint_id)): Path<(Uuid, Uuid, Uuid)>,
  Json(body): Json<UpdateViewpointRequest>,
) -> AppResult<Json<ViewpointResponse>> {
  // Decode and store new snapshot if provided
  let snapshot_path = if let Some(ref b64) = body.snapshot_data {
    let data = base64::engine::general_purpose::STANDARD
      .decode(b64)
      .map_err(|e| AppError::BadRequest(format!("invalid base64 snapshot: {e}")))?;

    let path = state
      .storage
      .save(topic_id, viewpoint_id, &data)
      .await
      .map_err(|e| AppError::Internal(format!("snapshot save failed: {e}")))?;
    Some(path)
  } else {
    None
  };

  let row = db::viewpoints::update_viewpoint(
    &state.pool,
    viewpoint_id,
    snapshot_path.as_deref(),
    body.camera.as_ref(),
    body.components.as_ref(),
  )
  .await?
  .ok_or_else(|| AppError::NotFound(format!("viewpoint {viewpoint_id}")))?;

  Ok(Json(row.into()))
}

/// DELETE /viewpoints/{viewpoint_id} — Delete a viewpoint and its snapshot.
async fn delete_viewpoint(
  State(state): State<AppState>,
  Path((_project_id, _topic_id, viewpoint_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<StatusCode> {
  // Get the viewpoint first to find snapshot path
  if let Some(row) = db::viewpoints::get_viewpoint(&state.pool, viewpoint_id).await? {
    if let Some(ref path) = row.snapshot_path {
      let _ = state.storage.delete(path).await;
    }
  }

  let deleted = db::viewpoints::delete_viewpoint(&state.pool, viewpoint_id).await?;
  if deleted {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(format!("viewpoint {viewpoint_id}")))
  }
}

/// GET /viewpoints/{viewpoint_id}/snapshot — Download snapshot as PNG.
async fn get_snapshot(
  State(state): State<AppState>,
  Path((_project_id, _topic_id, viewpoint_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
  let row = db::viewpoints::get_viewpoint(&state.pool, viewpoint_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("viewpoint {viewpoint_id}")))?;

  let path = row
    .snapshot_path
    .ok_or_else(|| AppError::NotFound("snapshot not available".to_string()))?;

  let data = state
    .storage
    .load(&path)
    .await
    .map_err(|e| AppError::Internal(format!("snapshot load failed: {e}")))?;

  Ok((
    [(header::CONTENT_TYPE, "image/png")],
    data,
  ))
}
