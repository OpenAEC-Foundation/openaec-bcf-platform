//! BCF ZIP import/export route handlers.
//!
//! POST /api/v1/projects/{id}/import-bcf — multipart upload .bcfzip
//! GET  /api/v1/projects/{id}/export-bcf  — download .bcfzip

use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use crate::db;
use crate::db::topics::CreateTopicParams;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Import/export routes (nested under /api/v1/projects).
pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/{project_id}/import-bcf", post(import_bcf))
    .route("/{project_id}/export-bcf", get(export_bcf))
}

/// Summary returned after a BCF import.
#[derive(Debug, Serialize)]
struct ImportSummary {
  topics_imported: usize,
  comments_imported: usize,
  viewpoints_imported: usize,
}

/// POST /import-bcf — Import a .bcfzip file into a project.
async fn import_bcf(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
  mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<ImportSummary>)> {
  // Verify project exists
  db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  // Read the uploaded file
  let mut file_data: Option<Vec<u8>> = None;
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
  {
    if field.name() == Some("file") {
      let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("failed to read file: {e}")))?;
      file_data = Some(data.to_vec());
      break;
    }
  }

  let data = file_data.ok_or_else(|| {
    AppError::BadRequest("missing 'file' field in multipart upload".to_string())
  })?;

  // Parse the BCF ZIP
  let archive = bcf_core::bcfzip::read_bcfzip(&data)
    .map_err(|e| AppError::BadRequest(format!("invalid BCF ZIP: {e}")))?;

  let mut total_topics = 0;
  let mut total_comments = 0;
  let mut total_viewpoints = 0;

  for folder in &archive.topics {
    // Create topic
    let labels = serde_json::to_value(&folder.topic.labels).unwrap_or_default();
    let params = CreateTopicParams {
      project_id,
      title: &folder.topic.title,
      description: &folder.topic.description,
      topic_type: &folder.topic.topic_type,
      topic_status: &folder.topic.topic_status,
      priority: &folder.topic.priority,
      assigned_to: None,
      stage: &folder.topic.stage,
      labels: &labels,
      due_date: folder.topic.due_date,
      index_number: folder.topic.index,
    };
    let topic_row = db::topics::create_topic(&state.pool, &params).await?;
    total_topics += 1;

    // Create comments
    for comment in &folder.comments {
      db::comments::create_comment(
        &state.pool,
        topic_row.id,
        &comment.comment,
        None,
      )
      .await?;
      total_comments += 1;
    }

    // Create viewpoints with snapshots (new UUIDs, not from BCF file)
    for vp in &folder.viewpoints {
      let camera_json = vp
        .camera
        .as_ref()
        .and_then(|c| serde_json::to_value(c).ok());
      let components_json = vp
        .components
        .as_ref()
        .and_then(|c| serde_json::to_value(c).ok());

      let vp_row = db::viewpoints::create_viewpoint(
        &state.pool,
        topic_row.id,
        None,
        camera_json.as_ref(),
        components_json.as_ref(),
      )
      .await?;

      // Save snapshot if present, using the new DB-generated ID
      if let Some(ref snap_data) = vp.snapshot_data {
        let path = state
          .storage
          .save(topic_row.id, vp_row.id, snap_data)
          .await
          .map_err(|e| AppError::Internal(format!("snapshot save failed: {e}")))?;
        db::viewpoints::update_viewpoint(
          &state.pool,
          vp_row.id,
          Some(&path),
          None,
          None,
        )
        .await?;
      }
      total_viewpoints += 1;
    }
  }

  tracing::info!(
    "imported BCF: {total_topics} topics, {total_comments} comments, {total_viewpoints} viewpoints into project {project_id}"
  );

  Ok((
    StatusCode::CREATED,
    Json(ImportSummary {
      topics_imported: total_topics,
      comments_imported: total_comments,
      viewpoints_imported: total_viewpoints,
    }),
  ))
}

/// GET /export-bcf — Export a project as .bcfzip download.
async fn export_bcf(
  State(state): State<AppState>,
  Path(project_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
  // Verify project exists
  let project = db::projects::get_project(&state.pool, project_id)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

  // Load all topics
  let topic_rows = db::topics::list_topics(&state.pool, project_id).await?;

  let mut topic_folders = Vec::new();

  for topic_row in &topic_rows {
    // Load comments
    let comment_rows = db::comments::list_comments(&state.pool, topic_row.id).await?;
    let comments: Vec<bcf_core::types::BcfComment> = comment_rows
      .into_iter()
      .map(|c| bcf_core::types::BcfComment {
        guid: c.id,
        comment: c.comment,
        author: None,
        viewpoint_guid: c.viewpoint_id,
        date: Some(c.created_at),
        modified_date: Some(c.updated_at),
        modified_author: None,
      })
      .collect();

    // Load viewpoints
    let vp_rows = db::viewpoints::list_viewpoints(&state.pool, topic_row.id).await?;
    let mut viewpoints = Vec::new();

    for vp_row in &vp_rows {
      let camera: Option<bcf_core::types::Camera> = vp_row
        .camera
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok());
      let components: Option<bcf_core::types::Components> = vp_row
        .components
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok());

      // Load snapshot from disk
      let snapshot_data = if let Some(ref path) = vp_row.snapshot_path {
        state.storage.load(path).await.ok()
      } else {
        None
      };

      viewpoints.push(bcf_core::types::BcfViewpoint {
        guid: vp_row.id,
        camera,
        components,
        snapshot_data,
      });
    }

    // Convert topic row to BcfTopic
    let labels: Vec<String> =
      serde_json::from_value(topic_row.labels.clone()).unwrap_or_default();

    let bcf_topic = bcf_core::types::BcfTopic {
      guid: topic_row.id,
      title: topic_row.title.clone(),
      description: topic_row.description.clone(),
      topic_type: topic_row.topic_type.clone(),
      topic_status: topic_row.topic_status.clone(),
      priority: topic_row.priority.clone(),
      stage: topic_row.stage.clone(),
      labels,
      due_date: topic_row.due_date,
      assigned_to: None,
      creation_author: None,
      modified_author: None,
      creation_date: Some(topic_row.created_at),
      modified_date: Some(topic_row.updated_at),
      index: topic_row.index_number,
    };

    topic_folders.push(bcf_core::bcfzip::BcfTopicFolder {
      topic: bcf_topic,
      comments,
      viewpoints,
    });
  }

  let archive = bcf_core::bcfzip::BcfArchive {
    version: "2.1".to_string(),
    topics: topic_folders,
  };

  let zip_bytes = bcf_core::bcfzip::write_bcfzip(&archive)
    .map_err(|e| AppError::Internal(format!("BCF ZIP generation failed: {e}")))?;

  let filename = format!(
    "{}.bcfzip",
    project.name.replace(' ', "_").to_lowercase()
  );

  Ok((
    [
      (header::CONTENT_TYPE, "application/zip".to_string()),
      (
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{filename}\""),
      ),
    ],
    zip_bytes,
  ))
}
