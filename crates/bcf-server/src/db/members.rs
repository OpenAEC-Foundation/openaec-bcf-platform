//! Project member database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::member::MemberRow;

/// List all members of a project (joined with users).
pub async fn list_members(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<Vec<MemberRow>, sqlx::Error> {
  sqlx::query_as::<_, MemberRow>(
    "SELECT pm.project_id, pm.user_id, pm.role, pm.created_at,
            u.email, u.name
     FROM project_members pm
     JOIN users u ON u.id = pm.user_id
     WHERE pm.project_id = $1
     ORDER BY pm.created_at",
  )
  .bind(project_id)
  .fetch_all(pool)
  .await
}

/// Add a member to a project.
pub async fn add_member(
  pool: &PgPool,
  project_id: Uuid,
  user_id: Uuid,
  role: &str,
) -> Result<MemberRow, sqlx::Error> {
  sqlx::query_as::<_, MemberRow>(
    "WITH ins AS (
       INSERT INTO project_members (project_id, user_id, role)
       VALUES ($1, $2, $3)
       RETURNING project_id, user_id, role, created_at
     )
     SELECT ins.project_id, ins.user_id, ins.role, ins.created_at,
            u.email, u.name
     FROM ins
     JOIN users u ON u.id = ins.user_id",
  )
  .bind(project_id)
  .bind(user_id)
  .bind(role)
  .fetch_one(pool)
  .await
}

/// Update a member's role.
pub async fn update_role(
  pool: &PgPool,
  project_id: Uuid,
  user_id: Uuid,
  role: &str,
) -> Result<Option<MemberRow>, sqlx::Error> {
  sqlx::query_as::<_, MemberRow>(
    "WITH upd AS (
       UPDATE project_members SET role = $3
       WHERE project_id = $1 AND user_id = $2
       RETURNING project_id, user_id, role, created_at
     )
     SELECT upd.project_id, upd.user_id, upd.role, upd.created_at,
            u.email, u.name
     FROM upd
     JOIN users u ON u.id = upd.user_id",
  )
  .bind(project_id)
  .bind(user_id)
  .bind(role)
  .fetch_optional(pool)
  .await
}

/// Remove a member from a project.
pub async fn remove_member(
  pool: &PgPool,
  project_id: Uuid,
  user_id: Uuid,
) -> Result<bool, sqlx::Error> {
  let result = sqlx::query(
    "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2",
  )
  .bind(project_id)
  .bind(user_id)
  .execute(pool)
  .await?;
  Ok(result.rows_affected() > 0)
}

/// Get a user's role in a project. Returns None if not a member.
#[allow(dead_code)]
pub async fn get_role(
  pool: &PgPool,
  project_id: Uuid,
  user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
  let row: Option<(String,)> = sqlx::query_as(
    "SELECT role FROM project_members WHERE project_id = $1 AND user_id = $2",
  )
  .bind(project_id)
  .bind(user_id)
  .fetch_optional(pool)
  .await?;
  Ok(row.map(|r| r.0))
}
