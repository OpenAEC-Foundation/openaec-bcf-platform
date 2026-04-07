//! Project extensions model (BCF v2.1).

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Database row for project extensions.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct ExtensionRow {
  pub id: Uuid,
  pub project_id: Uuid,
  pub topic_types: serde_json::Value,
  pub topic_statuses: serde_json::Value,
  pub priorities: serde_json::Value,
  pub labels: serde_json::Value,
  pub stages: serde_json::Value,
  pub updated_at: DateTime<Utc>,
}

/// API response for project extensions (BCF v2.1 compatible).
#[derive(Debug, Serialize)]
pub struct ExtensionResponse {
  pub topic_type: Vec<String>,
  pub topic_status: Vec<String>,
  pub priority: Vec<String>,
  pub topic_label: Vec<String>,
  pub stage: Vec<String>,
}

impl From<ExtensionRow> for ExtensionResponse {
  fn from(row: ExtensionRow) -> Self {
    Self {
      topic_type: serde_json::from_value(row.topic_types).unwrap_or_default(),
      topic_status: serde_json::from_value(row.topic_statuses).unwrap_or_default(),
      priority: serde_json::from_value(row.priorities).unwrap_or_default(),
      topic_label: serde_json::from_value(row.labels).unwrap_or_default(),
      stage: serde_json::from_value(row.stages).unwrap_or_default(),
    }
  }
}
