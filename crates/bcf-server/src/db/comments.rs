//! Comment database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::comment::CommentRow;

/// List all comments for a topic.
pub async fn list_comments(
  pool: &PgPool,
  topic_id: Uuid,
) -> Result<Vec<CommentRow>, sqlx::Error> {
  sqlx::query_as::<_, CommentRow>(
    "SELECT id, topic_id, author_id, comment, viewpoint_id, created_at, updated_at
     FROM comments
     WHERE topic_id = $1
     ORDER BY created_at ASC",
  )
  .bind(topic_id)
  .fetch_all(pool)
  .await
}

/// Get a single comment by ID.
pub async fn get_comment(
  pool: &PgPool,
  comment_id: Uuid,
) -> Result<Option<CommentRow>, sqlx::Error> {
  sqlx::query_as::<_, CommentRow>(
    "SELECT id, topic_id, author_id, comment, viewpoint_id, created_at, updated_at
     FROM comments
     WHERE id = $1",
  )
  .bind(comment_id)
  .fetch_optional(pool)
  .await
}

/// Insert a new comment.
pub async fn create_comment(
  pool: &PgPool,
  topic_id: Uuid,
  comment: &str,
  viewpoint_id: Option<Uuid>,
) -> Result<CommentRow, sqlx::Error> {
  sqlx::query_as::<_, CommentRow>(
    "INSERT INTO comments (topic_id, comment, viewpoint_id)
     VALUES ($1, $2, $3)
     RETURNING id, topic_id, author_id, comment, viewpoint_id, created_at, updated_at",
  )
  .bind(topic_id)
  .bind(comment)
  .bind(viewpoint_id)
  .fetch_one(pool)
  .await
}

/// Update an existing comment. Returns None if not found.
pub async fn update_comment(
  pool: &PgPool,
  comment_id: Uuid,
  comment: Option<&str>,
  viewpoint_id: Option<Uuid>,
) -> Result<Option<CommentRow>, sqlx::Error> {
  sqlx::query_as::<_, CommentRow>(
    "UPDATE comments
     SET comment = COALESCE($2, comment),
         viewpoint_id = COALESCE($3, viewpoint_id),
         updated_at = now()
     WHERE id = $1
     RETURNING id, topic_id, author_id, comment, viewpoint_id, created_at, updated_at",
  )
  .bind(comment_id)
  .bind(comment)
  .bind(viewpoint_id)
  .fetch_optional(pool)
  .await
}

/// Delete a comment. Returns true if deleted.
pub async fn delete_comment(pool: &PgPool, comment_id: Uuid) -> Result<bool, sqlx::Error> {
  let result = sqlx::query("DELETE FROM comments WHERE id = $1")
    .bind(comment_id)
    .execute(pool)
    .await?;
  Ok(result.rows_affected() > 0)
}
