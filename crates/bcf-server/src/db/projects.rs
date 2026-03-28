//! Project database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project::ProjectRow;

const SELECT_COLS: &str =
  "id, name, description, location, image_path, created_by, created_at, updated_at";

/// List all projects.
pub async fn list_projects(pool: &PgPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
  let q = format!("SELECT {SELECT_COLS} FROM projects ORDER BY updated_at DESC");
  sqlx::query_as::<_, ProjectRow>(&q)
    .fetch_all(pool)
    .await
}

/// Get a single project by ID.
pub async fn get_project(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<Option<ProjectRow>, sqlx::Error> {
  let q = format!("SELECT {SELECT_COLS} FROM projects WHERE id = $1");
  sqlx::query_as::<_, ProjectRow>(&q)
    .bind(project_id)
    .fetch_optional(pool)
    .await
}

/// Insert a new project.
pub async fn create_project(
  pool: &PgPool,
  name: &str,
  description: &str,
  location: &str,
  created_by: Option<Uuid>,
) -> Result<ProjectRow, sqlx::Error> {
  let q = format!(
    "INSERT INTO projects (name, description, location, created_by)
     VALUES ($1, $2, $3, $4)
     RETURNING {SELECT_COLS}"
  );
  sqlx::query_as::<_, ProjectRow>(&q)
    .bind(name)
    .bind(description)
    .bind(location)
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
  location: Option<&str>,
) -> Result<Option<ProjectRow>, sqlx::Error> {
  let q = format!(
    "UPDATE projects
     SET name = COALESCE($2, name),
         description = COALESCE($3, description),
         location = COALESCE($4, location),
         updated_at = now()
     WHERE id = $1
     RETURNING {SELECT_COLS}"
  );
  sqlx::query_as::<_, ProjectRow>(&q)
    .bind(project_id)
    .bind(name)
    .bind(description)
    .bind(location)
    .fetch_optional(pool)
    .await
}

/// Update a project's image path.
pub async fn set_image_path(
  pool: &PgPool,
  project_id: Uuid,
  image_path: &str,
) -> Result<Option<ProjectRow>, sqlx::Error> {
  let q = format!(
    "UPDATE projects SET image_path = $2, updated_at = now()
     WHERE id = $1
     RETURNING {SELECT_COLS}"
  );
  sqlx::query_as::<_, ProjectRow>(&q)
    .bind(project_id)
    .bind(image_path)
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
