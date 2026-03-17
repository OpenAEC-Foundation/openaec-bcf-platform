//! Project-related API and database models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for a project.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct ProjectRow {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub created_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// API response for a project.
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
  pub project_id: Uuid,
  pub name: String,
  #[serde(skip_serializing_if = "String::is_empty")]
  pub description: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<ProjectRow> for ProjectResponse {
  fn from(row: ProjectRow) -> Self {
    Self {
      project_id: row.id,
      name: row.name,
      description: row.description,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }
  }
}

/// Request body for creating a project.
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
  pub name: String,
  #[serde(default)]
  pub description: String,
}

/// Request body for updating a project.
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
  pub name: Option<String>,
  pub description: Option<String>,
}
