//! API key database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::api_key::ApiKeyRow;

/// Create a new API key.
pub async fn create_api_key(
  pool: &PgPool,
  project_id: Uuid,
  name: &str,
  key_hash: &str,
  prefix: &str,
  created_by: Option<Uuid>,
  expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<ApiKeyRow, sqlx::Error> {
  sqlx::query_as::<_, ApiKeyRow>(
    "INSERT INTO api_keys (project_id, name, key_hash, prefix, created_by, expires_at)
     VALUES ($1, $2, $3, $4, $5, $6)
     RETURNING id, project_id, name, key_hash, prefix, created_by, expires_at, created_at",
  )
  .bind(project_id)
  .bind(name)
  .bind(key_hash)
  .bind(prefix)
  .bind(created_by)
  .bind(expires_at)
  .fetch_one(pool)
  .await
}

/// List all API keys for a project (does not return hashes via model).
pub async fn list_api_keys(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<Vec<ApiKeyRow>, sqlx::Error> {
  sqlx::query_as::<_, ApiKeyRow>(
    "SELECT id, project_id, name, key_hash, prefix, created_by, expires_at, created_at
     FROM api_keys
     WHERE project_id = $1
     ORDER BY created_at DESC",
  )
  .bind(project_id)
  .fetch_all(pool)
  .await
}

/// Find API keys by prefix (for key validation).
pub async fn find_by_prefix(
  pool: &PgPool,
  prefix: &str,
) -> Result<Vec<ApiKeyRow>, sqlx::Error> {
  sqlx::query_as::<_, ApiKeyRow>(
    "SELECT id, project_id, name, key_hash, prefix, created_by, expires_at, created_at
     FROM api_keys
     WHERE prefix = $1",
  )
  .bind(prefix)
  .fetch_all(pool)
  .await
}

/// Delete an API key by ID (scoped to project).
pub async fn delete_api_key(
  pool: &PgPool,
  project_id: Uuid,
  key_id: Uuid,
) -> Result<bool, sqlx::Error> {
  let result = sqlx::query("DELETE FROM api_keys WHERE id = $1 AND project_id = $2")
    .bind(key_id)
    .bind(project_id)
    .execute(pool)
    .await?;
  Ok(result.rows_affected() > 0)
}
