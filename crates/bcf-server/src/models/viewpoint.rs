//! Viewpoint API and database models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for a viewpoint.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct ViewpointRow {
  pub id: Uuid,
  pub topic_id: Uuid,
  pub snapshot_path: Option<String>,
  pub camera: Option<serde_json::Value>,
  pub components: Option<serde_json::Value>,
  pub created_at: DateTime<Utc>,
}

/// API response for a viewpoint (BCF v2.1 compatible).
#[derive(Debug, Serialize)]
pub struct ViewpointResponse {
  pub guid: Uuid,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub camera: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub components: Option<serde_json::Value>,
  pub has_snapshot: bool,
  pub creation_date: DateTime<Utc>,
}

impl From<ViewpointRow> for ViewpointResponse {
  fn from(row: ViewpointRow) -> Self {
    Self {
      guid: row.id,
      camera: row.camera,
      components: row.components,
      has_snapshot: row.snapshot_path.is_some(),
      creation_date: row.created_at,
    }
  }
}

/// Request body for creating a viewpoint.
#[derive(Debug, Deserialize)]
pub struct CreateViewpointRequest {
  pub camera: Option<serde_json::Value>,
  pub components: Option<serde_json::Value>,
  /// Base64-encoded PNG snapshot data.
  pub snapshot_data: Option<String>,
}

/// Request body for updating a viewpoint.
#[derive(Debug, Deserialize)]
pub struct UpdateViewpointRequest {
  pub camera: Option<serde_json::Value>,
  pub components: Option<serde_json::Value>,
  /// Base64-encoded PNG snapshot data (replaces existing).
  pub snapshot_data: Option<String>,
}
