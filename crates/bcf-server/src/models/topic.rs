//! Topic (issue) API and database models.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for a topic.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct TopicRow {
  pub id: Uuid,
  pub project_id: Uuid,
  pub title: String,
  pub description: String,
  pub topic_type: String,
  pub topic_status: String,
  pub priority: String,
  pub assigned_to: Option<Uuid>,
  pub stage: String,
  pub labels: serde_json::Value,
  pub due_date: Option<NaiveDate>,
  pub creation_author: Option<Uuid>,
  pub modified_author: Option<Uuid>,
  pub index_number: Option<i32>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// API response for a topic (BCF v2.1 compatible field names).
#[derive(Debug, Serialize)]
pub struct TopicResponse {
  pub guid: Uuid,
  pub title: String,
  #[serde(skip_serializing_if = "String::is_empty")]
  pub description: String,
  pub topic_type: String,
  pub topic_status: String,
  pub priority: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub assigned_to: Option<Uuid>,
  #[serde(skip_serializing_if = "String::is_empty")]
  pub stage: String,
  pub labels: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub due_date: Option<NaiveDate>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub index: Option<i32>,
  pub creation_date: DateTime<Utc>,
  pub modified_date: DateTime<Utc>,
}

impl From<TopicRow> for TopicResponse {
  fn from(row: TopicRow) -> Self {
    let labels: Vec<String> = serde_json::from_value(row.labels).unwrap_or_default();
    Self {
      guid: row.id,
      title: row.title,
      description: row.description,
      topic_type: row.topic_type,
      topic_status: row.topic_status,
      priority: row.priority,
      assigned_to: row.assigned_to,
      stage: row.stage,
      labels,
      due_date: row.due_date,
      index: row.index_number,
      creation_date: row.created_at,
      modified_date: row.updated_at,
    }
  }
}

/// Request body for creating a topic.
#[derive(Debug, Deserialize)]
pub struct CreateTopicRequest {
  pub title: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub topic_type: String,
  #[serde(default = "default_status")]
  pub topic_status: String,
  #[serde(default = "default_priority")]
  pub priority: String,
  pub assigned_to: Option<Uuid>,
  #[serde(default)]
  pub stage: String,
  #[serde(default)]
  pub labels: Vec<String>,
  pub due_date: Option<NaiveDate>,
  pub index: Option<i32>,
}

fn default_status() -> String {
  "Open".to_string()
}

fn default_priority() -> String {
  "Normal".to_string()
}

/// Request body for updating a topic.
#[derive(Debug, Deserialize)]
pub struct UpdateTopicRequest {
  pub title: Option<String>,
  pub description: Option<String>,
  pub topic_type: Option<String>,
  pub topic_status: Option<String>,
  pub priority: Option<String>,
  pub assigned_to: Option<Uuid>,
  pub stage: Option<String>,
  pub labels: Option<Vec<String>>,
  pub due_date: Option<NaiveDate>,
  pub index: Option<i32>,
}
