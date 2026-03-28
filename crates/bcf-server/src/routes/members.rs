//! Project member management routes.
//!
//! GET    /api/v1/projects/{id}/members              — list members
//! POST   /api/v1/projects/{id}/members              — add member
//! PUT    /api/v1/projects/{id}/members/{user_id}    — update role
//! DELETE /api/v1/projects/{id}/members/{user_id}    — remove member

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::member::{AddMemberRequest, MemberResponse, UpdateMemberRequest};
use crate::state::AppState;

const VALID_ROLES: &[&str] = &["owner", "admin", "member", "viewer"];

/// Member routes (nested under /api/v1/projects).
pub fn routes() -> Router<AppState> {
  Router::new()
    .route(
      "/{project_id}/members",
      get(list_members).post(add_member),
    )
    .route(
      "/{project_id}/members/{user_id}",
      axum::routing::put(update_role).delete(remove_member),
    )
}

/// GET /members — List all members of a project.
async fn list_members(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<Vec<MemberResponse>>> {
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  let rows = db::members::list_members(&state.pool, project_id).await?;
  let members: Vec<MemberResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(members))
}

/// POST /members — Add a member to a project.
async fn add_member(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
  Json(body): Json<AddMemberRequest>,
) -> AppResult<(StatusCode, Json<MemberResponse>)> {
  if !VALID_ROLES.contains(&body.role.as_str()) {
    return Err(AppError::BadRequest(format!(
      "invalid role '{}', must be one of: {}",
      body.role,
      VALID_ROLES.join(", ")
    )));
  }

  // Verify project and user exist
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;
  db::users::find_by_id(&state.pool, body.user_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("user {}", body.user_id)))?;

  let row = db::members::add_member(&state.pool, project_id, body.user_id, &body.role).await?;
  Ok((StatusCode::CREATED, Json(row.into())))
}

/// PUT /members/{user_id} — Update a member's role.
async fn update_role(
  State(state): State<AppState>,
  Path((project_id, user_id)): Path<(Uuid, Uuid)>,
  Json(body): Json<UpdateMemberRequest>,
) -> AppResult<Json<MemberResponse>> {
  if !VALID_ROLES.contains(&body.role.as_str()) {
    return Err(AppError::BadRequest(format!(
      "invalid role '{}', must be one of: {}",
      body.role,
      VALID_ROLES.join(", ")
    )));
  }

  let row = db::members::update_role(&state.pool, project_id, user_id, &body.role)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("member {user_id} in project {project_id}")))?;
  Ok(Json(row.into()))
}

/// DELETE /members/{user_id} — Remove a member from a project.
async fn remove_member(
  State(state): State<AppState>,
  Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
  let removed = db::members::remove_member(&state.pool, project_id, user_id).await?;
  if removed {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(format!(
      "member {user_id} in project {project_id}"
    )))
  }
}
