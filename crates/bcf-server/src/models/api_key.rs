//! API key models for service-to-service authentication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for an API key.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiKeyRow {
  pub id: Uuid,
  pub project_id: Uuid,
  pub name: String,
  pub key_hash: String,
  pub prefix: String,
  pub created_by: Option<Uuid>,
  pub expires_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
}

/// API response for an API key (never includes the hash).
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
  pub id: Uuid,
  pub project_id: Uuid,
  pub name: String,
  pub prefix: String,
  pub created_by: Option<Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expires_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
}

impl From<ApiKeyRow> for ApiKeyResponse {
  fn from(row: ApiKeyRow) -> Self {
    Self {
      id: row.id,
      project_id: row.project_id,
      name: row.name,
      prefix: row.prefix,
      created_by: row.created_by,
      expires_at: row.expires_at,
      created_at: row.created_at,
    }
  }
}

/// Response returned when creating an API key (includes the raw key, shown once).
#[derive(Debug, Serialize)]
pub struct ApiKeyCreatedResponse {
  pub id: Uuid,
  pub name: String,
  pub prefix: String,
  /// The raw API key — shown only once at creation time.
  pub key: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expires_at: Option<DateTime<Utc>>,
}

/// Request body for creating an API key.
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
  pub name: String,
  /// Optional expiration timestamp.
  pub expires_at: Option<DateTime<Utc>>,
}
