//! Project CRUD route handlers.
//!
//! BCF v2.1 endpoints: GET/POST /bcf/2.1/projects, GET/PUT /bcf/2.1/projects/{id}
//! Platform endpoints: POST/DELETE /api/v1/projects/{id}

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::project::{
  CreateProjectRequest, ProjectResponse, UpdateProjectRequest,
};
use crate::state::AppState;

use super::topics;

/// BCF v2.1 project routes (nested under /bcf/2.1/projects).
pub fn bcf_project_routes() -> Router<AppState> {
  Router::new()
    .route("/", get(list_projects).post(create_project))
    .route("/{project_id}", get(get_project).put(update_project))
    .nest("/{project_id}/topics", topics::routes())
}

/// Platform-specific project routes (nested under /api/v1/projects).
pub fn platform_project_routes() -> Router<AppState> {
  Router::new()
    .route("/", post(create_project))
    .route("/{project_id}", delete(delete_project))
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

/// POST /projects — Create a new project.
async fn create_project(
  State(state): State<AppState>,
  Json(body): Json<CreateProjectRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ProjectResponse>)> {
  if body.name.trim().is_empty() {
    return Err(AppError::BadRequest("name is required".to_string()));
  }
  let row = db::projects::create_project(&state.pool, &body.name, &body.description).await?;
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
  )
  .await?
  .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;
  Ok(Json(row.into()))
}

/// DELETE /api/v1/projects/{project_id} — Delete a project.
async fn delete_project(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<axum::http::StatusCode> {
  let deleted = db::projects::delete_project(&state.pool, project_id).await?;
  if deleted {
    Ok(axum::http::StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(format!("project {project_id}")))
  }
}
