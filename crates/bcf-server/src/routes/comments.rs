//! Comment CRUD route handlers.
//!
//! Nested under /bcf/2.1/projects/{project_id}/topics/{topic_id}/comments

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::OptionalAuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::comment::{CommentResponse, CreateCommentRequest, UpdateCommentRequest};
use crate::state::AppState;

/// Comment routes (nested under topics/{topic_id}/comments).
pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/", get(list_comments).post(create_comment))
    .route("/{comment_id}", get(get_comment).put(update_comment).delete(delete_comment))
}

/// GET /comments — List all comments for a topic.
async fn list_comments(
  State(state): State<AppState>,
  Path((_project_id, topic_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<CommentResponse>>> {
  let rows = db::comments::list_comments(&state.pool, topic_id).await?;
  let comments: Vec<CommentResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(comments))
}

/// GET /comments/{comment_id} — Get a single comment.
async fn get_comment(
  State(state): State<AppState>,
  Path((_project_id, _topic_id, comment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<CommentResponse>> {
  let row = db::comments::get_comment(&state.pool, comment_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("comment {comment_id}")))?;
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

/// POST /comments — Create a new comment.
async fn create_comment(
  State(state): State<AppState>,
  Path((_project_id, topic_id)): Path<(Uuid, Uuid)>,
  auth: OptionalAuthUser,
  Json(body): Json<CreateCommentRequest>,
) -> AppResult<(axum::http::StatusCode, Json<CommentResponse>)> {
  if body.comment.trim().is_empty() {
    return Err(AppError::BadRequest("comment is required".to_string()));
  }

  // Verify topic exists
  let topic_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM topics WHERE id = $1)")
    .bind(topic_id)
    .fetch_one(&state.pool)
    .await?;

  if !topic_exists {
    return Err(AppError::NotFound(format!("topic {topic_id}")));
  }

  let author_id = user_id_or_none(&auth);
  let row = db::comments::create_comment(
    &state.pool,
    topic_id,
    &body.comment,
    body.viewpoint_guid,
    author_id,
  )
  .await?;
  Ok((axum::http::StatusCode::CREATED, Json(row.into())))
}

/// PUT /comments/{comment_id} — Update an existing comment.
async fn update_comment(
  State(state): State<AppState>,
  Path((_project_id, _topic_id, comment_id)): Path<(Uuid, Uuid, Uuid)>,
  Json(body): Json<UpdateCommentRequest>,
) -> AppResult<Json<CommentResponse>> {
  let row = db::comments::update_comment(
    &state.pool,
    comment_id,
    body.comment.as_deref(),
    body.viewpoint_guid,
  )
  .await?
  .ok_or_else(|| AppError::NotFound(format!("comment {comment_id}")))?;
  Ok(Json(row.into()))
}

/// DELETE /comments/{comment_id} — Delete a comment.
async fn delete_comment(
  State(state): State<AppState>,
  Path((_project_id, _topic_id, comment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<axum::http::StatusCode> {
  let deleted = db::comments::delete_comment(&state.pool, comment_id).await?;
  if deleted {
    Ok(axum::http::StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(format!("comment {comment_id}")))
  }
}
