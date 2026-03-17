//! Comment API and database models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for a comment.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct CommentRow {
  pub id: Uuid,
  pub topic_id: Uuid,
  pub author_id: Option<Uuid>,
  pub comment: String,
  pub viewpoint_id: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// API response for a comment (BCF v2.1 compatible).
#[derive(Debug, Serialize)]
pub struct CommentResponse {
  pub guid: Uuid,
  pub comment: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author_id: Option<Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub viewpoint_guid: Option<Uuid>,
  pub date: DateTime<Utc>,
  pub modified_date: DateTime<Utc>,
}

impl From<CommentRow> for CommentResponse {
  fn from(row: CommentRow) -> Self {
    Self {
      guid: row.id,
      comment: row.comment,
      author_id: row.author_id,
      viewpoint_guid: row.viewpoint_id,
      date: row.created_at,
      modified_date: row.updated_at,
    }
  }
}

/// Request body for creating a comment.
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
  pub comment: String,
  pub viewpoint_guid: Option<Uuid>,
}

/// Request body for updating a comment.
#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
  pub comment: Option<String>,
  pub viewpoint_guid: Option<Uuid>,
}
