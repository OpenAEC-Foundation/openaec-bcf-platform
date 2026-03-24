//! Project database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project::ProjectRow;

/// List all projects.
pub async fn list_projects(pool: &PgPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "SELECT id, name, description, created_by, created_at, updated_at
     FROM projects
     ORDER BY updated_at DESC",
  )
  .fetch_all(pool)
  .await
}

/// Get a single project by ID.
pub async fn get_project(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<Option<ProjectRow>, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "SELECT id, name, description, created_by, created_at, updated_at
     FROM projects
     WHERE id = $1",
  )
  .bind(project_id)
  .fetch_optional(pool)
  .await
}

/// Insert a new project.
pub async fn create_project(
  pool: &PgPool,
  name: &str,
  description: &str,
  created_by: Option<Uuid>,
) -> Result<ProjectRow, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "INSERT INTO projects (name, description, created_by)
     VALUES ($1, $2, $3)
     RETURNING id, name, description, created_by, created_at, updated_at",
  )
  .bind(name)
  .bind(description)
  .bind(created_by)
  .fetch_one(pool)
  .await
}

/// Update an existing project. Returns None if not found.
pub async fn update_project(
  pool: &PgPool,
  project_id: Uuid,
  name: Option<&str>,
  description: Option<&str>,
) -> Result<Option<ProjectRow>, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "UPDATE projects
     SET name = COALESCE($2, name),
         description = COALESCE($3, description),
         updated_at = now()
     WHERE id = $1
     RETURNING id, name, description, created_by, created_at, updated_at",
  )
  .bind(project_id)
  .bind(name)
  .bind(description)
  .fetch_optional(pool)
  .await
}

/// Delete a project by ID. Returns true if deleted.
pub async fn delete_project(pool: &PgPool, project_id: Uuid) -> Result<bool, sqlx::Error> {
  let result = sqlx::query("DELETE FROM projects WHERE id = $1")
    .bind(project_id)
    .execute(pool)
    .await?;
  Ok(result.rows_affected() > 0)
}
