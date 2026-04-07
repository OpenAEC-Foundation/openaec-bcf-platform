//! Project CRUD route handlers.
//!
//! BCF v2.1 endpoints: GET/POST /bcf/2.1/projects, GET/PUT /bcf/2.1/projects/{id}
//! Platform endpoints: POST/DELETE /api/v1/projects/{id}, image upload/serve, stats

use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::OptionalAuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::project::{
  CreateProjectRequest, ProjectResponse, UpdateProjectRequest,
};
use crate::state::AppState;

use super::{events, extensions, topics};

/// BCF v2.1 project routes (nested under /bcf/2.1/projects).
pub fn bcf_project_routes() -> Router<AppState> {
  Router::new()
    .route("/", get(list_projects).post(create_project))
    .route("/{project_id}", get(get_project).put(update_project))
    .nest("/{project_id}/extensions", extensions::routes())
    .nest("/{project_id}/events", events::project_event_routes())
    .nest("/{project_id}/topics", topics::routes())
}

/// Platform-specific project routes (nested under /api/v1/projects).
pub fn platform_project_routes() -> Router<AppState> {
  Router::new()
    .route("/", post(create_project))
    .route("/{project_id}", delete(delete_project))
    .route("/{project_id}/image", put(upload_image).get(get_image))
    .route("/{project_id}/stats", get(project_stats))
}

/// GET /projects — List all projects.
async fn list_projects(State(state): State<AppState>) -> AppResult<Json<Vec<ProjectResponse>>> {
  let rows = db::projects::list_projects(&state.pool).await?;
  let projects: Vec<ProjectResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(projects))
}

/// GET /projects/{project_id} — Get a single project.
async fn get_project(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<ProjectResponse>> {
  let row = db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;
  Ok(Json(row.into()))
}

/// Helper to convert an optional auth user to a database-friendly Option<Uuid>.
fn user_id_or_none(auth: &OptionalAuthUser) -> Option<Uuid> {
  auth
    .0
    .as_ref()
    .map(|u| u.user_id)
    .filter(|id| !id.is_nil())
}

/// POST /projects — Create a new project.
async fn create_project(
  State(state): State<AppState>,
  auth: OptionalAuthUser,
  Json(body): Json<CreateProjectRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ProjectResponse>)> {
  if body.name.trim().is_empty() {
    return Err(AppError::BadRequest("name is required".to_string()));
  }
  let created_by = user_id_or_none(&auth);
  let row = db::projects::create_project(
    &state.pool,
    &body.name,
    &body.description,
    &body.location,
    created_by,
  )
  .await?;
  Ok((axum::http::StatusCode::CREATED, Json(row.into())))
}

/// PUT /projects/{project_id} — Update an existing project.
async fn update_project(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
  Json(body): Json<UpdateProjectRequest>,
) -> AppResult<Json<ProjectResponse>> {
  let row = db::projects::update_project(
    &state.pool,
    project_id,
    body.name.as_deref(),
    body.description.as_deref(),
    body.location.as_deref(),
  )
  .await?
  .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;
  Ok(Json(row.into()))
}

/// DELETE /api/v1/projects/{project_id} — Delete a project.
async fn delete_project(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<StatusCode> {
  let deleted = db::projects::delete_project(&state.pool, project_id).await?;
  if deleted {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(format!("project {project_id}")))
  }
}

/// PUT /api/v1/projects/{project_id}/image — Upload project image.
async fn upload_image(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
  mut multipart: Multipart,
) -> AppResult<Json<ProjectResponse>> {
  // Verify project exists
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  let mut file_data: Option<Vec<u8>> = None;
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
  {
    if field.name() == Some("image") {
      let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("read error: {e}")))?;
      file_data = Some(data.to_vec());
      break;
    }
  }

  let data = file_data.ok_or_else(|| {
    AppError::BadRequest("missing 'image' field".to_string())
  })?;

  // Save to storage
  let dir = std::path::Path::new(&state.config.storage_path).join("project-images");
  tokio::fs::create_dir_all(&dir)
    .await
    .map_err(|e| AppError::Internal(format!("mkdir failed: {e}")))?;

  let filename = format!("{project_id}.img");
  let path = dir.join(&filename);
  tokio::fs::write(&path, &data)
    .await
    .map_err(|e| AppError::Internal(format!("write failed: {e}")))?;

  let rel_path = format!("project-images/{filename}");
  let row = db::projects::set_image_path(&state.pool, project_id, &rel_path)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  Ok(Json(row.into()))
}

/// GET /api/v1/projects/{project_id}/image — Serve project image.
async fn get_image(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
  let project = db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  let image_path = project
    .image_path
    .ok_or_else(|| AppError::NotFound("no image".to_string()))?;

  let full_path = std::path::Path::new(&state.config.storage_path).join(&image_path);
  let data = tokio::fs::read(&full_path)
    .await
    .map_err(|_| AppError::NotFound("image file missing".to_string()))?;

  Ok((
    [(header::CONTENT_TYPE, "image/png".to_string())],
    Body::from(data),
  ))
}

/// Dashboard stats for a project.
#[derive(Serialize)]
struct ProjectStats {
  total: i64,
  open: i64,
  in_progress: i64,
  closed: i64,
  by_priority: Vec<PriorityCount>,
}

#[derive(Serialize)]
struct PriorityCount {
  priority: String,
  count: i64,
}

/// GET /api/v1/projects/{project_id}/stats — Project dashboard statistics.
async fn project_stats(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<ProjectStats>> {
  // Verify project exists
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  // Count by status
  let status_counts: Vec<(String, i64)> = sqlx::query_as(
    "SELECT topic_status, COUNT(*) FROM topics WHERE project_id = $1 GROUP BY topic_status",
  )
  .bind(project_id)
  .fetch_all(&state.pool)
  .await?;

  let mut total: i64 = 0;
  let mut open: i64 = 0;
  let mut in_progress: i64 = 0;
  let mut closed: i64 = 0;

  for (status, count) in &status_counts {
    total += count;
    match status.to_lowercase().as_str() {
      "open" | "reopened" => open += count,
      "active" | "in progress" => in_progress += count,
      "closed" | "resolved" => closed += count,
      _ => open += count,
    }
  }

  // Count by priority
  let by_priority: Vec<PriorityCount> = sqlx::query_as::<_, (String, i64)>(
    "SELECT priority, COUNT(*) FROM topics WHERE project_id = $1 GROUP BY priority ORDER BY COUNT(*) DESC",
  )
  .bind(project_id)
  .fetch_all(&state.pool)
  .await?
  .into_iter()
  .map(|(priority, count)| PriorityCount { priority, count })
  .collect();

  Ok(Json(ProjectStats {
    total,
    open,
    in_progress,
    closed,
    by_priority,
  }))
}
