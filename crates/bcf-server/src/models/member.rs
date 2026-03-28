//! Project member models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for a project member.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct MemberRow {
  pub project_id: Uuid,
  pub user_id: Uuid,
  pub role: String,
  pub created_at: DateTime<Utc>,
  /// Joined from users table.
  pub email: String,
  pub name: String,
}

/// API response for a project member.
#[derive(Debug, Serialize)]
pub struct MemberResponse {
  pub user_id: Uuid,
  pub email: String,
  pub name: String,
  pub role: String,
  pub created_at: DateTime<Utc>,
}

impl From<MemberRow> for MemberResponse {
  fn from(row: MemberRow) -> Self {
    Self {
      user_id: row.user_id,
      email: row.email,
      name: row.name,
      role: row.role,
      created_at: row.created_at,
    }
  }
}

/// Request body for adding a member.
#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
  pub user_id: Uuid,
  pub role: String,
}

/// Request body for updating a member's role.
#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
  pub role: String,
}
