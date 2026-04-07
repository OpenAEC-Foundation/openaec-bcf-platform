//! Event (audit log) database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::event::EventRow;

/// List events for a specific topic.
pub async fn list_topic_events(
  pool: &PgPool,
  topic_id: Uuid,
) -> Result<Vec<EventRow>, sqlx::Error> {
  sqlx::query_as::<_, EventRow>(
    "SELECT id, topic_id, author_id, event_type, old_value, new_value, created_at
     FROM events
     WHERE topic_id = $1
     ORDER BY created_at ASC",
  )
  .bind(topic_id)
  .fetch_all(pool)
  .await
}

/// List events for all topics in a project.
pub async fn list_project_events(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<Vec<EventRow>, sqlx::Error> {
  sqlx::query_as::<_, EventRow>(
    "SELECT e.id, e.topic_id, e.author_id, e.event_type, e.old_value, e.new_value, e.created_at
     FROM events e
     JOIN topics t ON t.id = e.topic_id
     WHERE t.project_id = $1
     ORDER BY e.created_at ASC",
  )
  .bind(project_id)
  .fetch_all(pool)
  .await
}
