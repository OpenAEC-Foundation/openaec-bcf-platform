//! Viewpoint database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::viewpoint::ViewpointRow;

/// List all viewpoints for a topic.
pub async fn list_viewpoints(
  pool: &PgPool,
  topic_id: Uuid,
) -> Result<Vec<ViewpointRow>, sqlx::Error> {
  sqlx::query_as::<_, ViewpointRow>(
    "SELECT id, topic_id, snapshot_path, camera, components, created_at
     FROM viewpoints
     WHERE topic_id = $1
     ORDER BY created_at ASC",
  )
  .bind(topic_id)
  .fetch_all(pool)
  .await
}

/// Get a single viewpoint by ID.
pub async fn get_viewpoint(
  pool: &PgPool,
  viewpoint_id: Uuid,
) -> Result<Option<ViewpointRow>, sqlx::Error> {
  sqlx::query_as::<_, ViewpointRow>(
    "SELECT id, topic_id, snapshot_path, camera, components, created_at
     FROM viewpoints
     WHERE id = $1",
  )
  .bind(viewpoint_id)
  .fetch_optional(pool)
  .await
}

/// Create a new viewpoint.
pub async fn create_viewpoint(
  pool: &PgPool,
  topic_id: Uuid,
  snapshot_path: Option<&str>,
  camera: Option<&serde_json::Value>,
  components: Option<&serde_json::Value>,
) -> Result<ViewpointRow, sqlx::Error> {
  sqlx::query_as::<_, ViewpointRow>(
    "INSERT INTO viewpoints (topic_id, snapshot_path, camera, components)
     VALUES ($1, $2, $3, $4)
     RETURNING id, topic_id, snapshot_path, camera, components, created_at",
  )
  .bind(topic_id)
  .bind(snapshot_path)
  .bind(camera)
  .bind(components)
  .fetch_one(pool)
  .await
}

/// Create a viewpoint with a specific ID (reserved for future use).
#[allow(dead_code)]
pub async fn create_viewpoint_with_id(
  pool: &PgPool,
  id: Uuid,
  topic_id: Uuid,
  snapshot_path: Option<&str>,
  camera: Option<&serde_json::Value>,
  components: Option<&serde_json::Value>,
) -> Result<ViewpointRow, sqlx::Error> {
  sqlx::query_as::<_, ViewpointRow>(
    "INSERT INTO viewpoints (id, topic_id, snapshot_path, camera, components)
     VALUES ($1, $2, $3, $4, $5)
     RETURNING id, topic_id, snapshot_path, camera, components, created_at",
  )
  .bind(id)
  .bind(topic_id)
  .bind(snapshot_path)
  .bind(camera)
  .bind(components)
  .fetch_one(pool)
  .await
}

/// Update an existing viewpoint.
pub async fn update_viewpoint(
  pool: &PgPool,
  viewpoint_id: Uuid,
  snapshot_path: Option<&str>,
  camera: Option<&serde_json::Value>,
  components: Option<&serde_json::Value>,
) -> Result<Option<ViewpointRow>, sqlx::Error> {
  sqlx::query_as::<_, ViewpointRow>(
    "UPDATE viewpoints
     SET snapshot_path = COALESCE($2, snapshot_path),
         camera = COALESCE($3, camera),
         components = COALESCE($4, components)
     WHERE id = $1
     RETURNING id, topic_id, snapshot_path, camera, components, created_at",
  )
  .bind(viewpoint_id)
  .bind(snapshot_path)
  .bind(camera)
  .bind(components)
  .fetch_optional(pool)
  .await
}

/// Delete a viewpoint.
pub async fn delete_viewpoint(
  pool: &PgPool,
  viewpoint_id: Uuid,
) -> Result<bool, sqlx::Error> {
  let result = sqlx::query("DELETE FROM viewpoints WHERE id = $1")
    .bind(viewpoint_id)
    .execute(pool)
    .await?;
  Ok(result.rows_affected() > 0)
}
