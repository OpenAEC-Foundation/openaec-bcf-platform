//! Topic (issue) CRUD route handlers.
//!
//! Nested under /bcf/2.1/projects/{project_id}/topics

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::db;
use crate::db::topics::{CreateTopicParams, UpdateTopicParams};
use crate::error::{AppError, AppResult};
use crate::models::topic::{CreateTopicRequest, TopicResponse, UpdateTopicRequest};
use crate::state::AppState;

use super::comments;
use super::viewpoints;

/// Topic routes (nested under projects/{project_id}/topics).
pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/", get(list_topics).post(create_topic))
    .route("/{topic_id}", get(get_topic).put(update_topic).delete(delete_topic))
    .nest("/{topic_id}/comments", comments::routes())
    .nest("/{topic_id}/viewpoints", viewpoints::routes())
}

/// GET /topics — List all topics for a project.
async fn list_topics(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<Vec<TopicResponse>>> {
  let rows = db::topics::list_topics(&state.pool, project_id).await?;
  let topics: Vec<TopicResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(topics))
}

/// GET /topics/{topic_id} — Get a single topic.
async fn get_topic(
  State(state): State<AppState>,
  Path((project_id, topic_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<TopicResponse>> {
  let row = db::topics::get_topic(&state.pool, project_id, topic_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("topic {topic_id}")))?;
  Ok(Json(row.into()))
}

/// POST /topics — Create a new topic.
async fn create_topic(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
  Json(body): Json<CreateTopicRequest>,
) -> AppResult<(axum::http::StatusCode, Json<TopicResponse>)> {
  if body.title.trim().is_empty() {
    return Err(AppError::BadRequest("title is required".to_string()));
  }

  // Verify project exists
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  let labels = serde_json::to_value(&body.labels).unwrap_or_default();
  let params = CreateTopicParams {
    project_id,
    title: &body.title,
    description: &body.description,
    topic_type: &body.topic_type,
    topic_status: &body.topic_status,
    priority: &body.priority,
    assigned_to: body.assigned_to,
    stage: &body.stage,
    labels: &labels,
    due_date: body.due_date,
    index_number: body.index,
  };
  let row = db::topics::create_topic(&state.pool, &params).await?;
  Ok((axum::http::StatusCode::CREATED, Json(row.into())))
}

/// PUT /topics/{topic_id} — Update an existing topic.
async fn update_topic(
  State(state): State<AppState>,
  Path((project_id, topic_id)): Path<(Uuid, Uuid)>,
  Json(body): Json<UpdateTopicRequest>,
) -> AppResult<Json<TopicResponse>> {
  let labels = body
    .labels
    .as_ref()
    .map(|l| serde_json::to_value(l).unwrap_or_default());
  let params = UpdateTopicParams {
    project_id,
    topic_id,
    title: body.title.as_deref(),
    description: body.description.as_deref(),
    topic_type: body.topic_type.as_deref(),
    topic_status: body.topic_status.as_deref(),
    priority: body.priority.as_deref(),
    assigned_to: body.assigned_to,
    stage: body.stage.as_deref(),
    labels: labels.as_ref(),
    due_date: body.due_date,
    index_number: body.index,
  };
  let row = db::topics::update_topic(&state.pool, &params)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("topic {topic_id}")))?;
  Ok(Json(row.into()))
}

/// DELETE /topics/{topic_id} — Delete a topic.
async fn delete_topic(
  State(state): State<AppState>,
  Path((project_id, topic_id)): Path<(Uuid, Uuid)>,
) -> AppResult<axum::http::StatusCode> {
  let deleted = db::topics::delete_topic(&state.pool, project_id, topic_id).await?;
  if deleted {
    Ok(axum::http::StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(format!("topic {topic_id}")))
  }
}
