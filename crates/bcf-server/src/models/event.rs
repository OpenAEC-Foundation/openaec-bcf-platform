//! Event (audit log) model (BCF v2.1).

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Database row for an event.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct EventRow {
  pub id: Uuid,
  pub topic_id: Uuid,
  pub author_id: Option<Uuid>,
  pub event_type: String,
  pub old_value: Option<String>,
  pub new_value: Option<String>,
  pub created_at: DateTime<Utc>,
}

/// API response for an event (BCF v2.1 compatible).
#[derive(Debug, Serialize)]
pub struct EventResponse {
  pub topic_guid: Uuid,
  pub date: DateTime<Utc>,
  #[serde(rename = "type")]
  pub event_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub old_value: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub new_value: Option<String>,
}

impl From<EventRow> for EventResponse {
  fn from(row: EventRow) -> Self {
    Self {
      topic_guid: row.topic_id,
      date: row.created_at,
      event_type: row.event_type,
      author: row.author_id,
      old_value: row.old_value,
      new_value: row.new_value,
    }
  }
}
