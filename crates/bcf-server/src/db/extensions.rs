//! Project extensions database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::extension::ExtensionRow;

/// Get project extensions, creating default row if none exists.
pub async fn get_or_create_extensions(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<ExtensionRow, sqlx::Error> {
  let row = sqlx::query_as::<_, ExtensionRow>(
    "INSERT INTO project_extensions (project_id)
     VALUES ($1)
     ON CONFLICT (project_id) DO UPDATE
       SET updated_at = project_extensions.updated_at
     RETURNING id, project_id, topic_types, topic_statuses, priorities, labels, stages, updated_at",
  )
  .bind(project_id)
  .fetch_one(pool)
  .await?;

  Ok(row)
}
