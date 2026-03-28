//! Topic database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::topic::TopicRow;

/// Parameters for creating a topic.
pub struct CreateTopicParams<'a> {
  pub project_id: Uuid,
  pub title: &'a str,
  pub description: &'a str,
  pub topic_type: &'a str,
  pub topic_status: &'a str,
  pub priority: &'a str,
  pub assigned_to: Option<Uuid>,
  pub stage: &'a str,
  pub labels: &'a serde_json::Value,
  pub due_date: Option<chrono::NaiveDate>,
  pub index_number: Option<i32>,
  pub creation_author: Option<Uuid>,
}

/// Parameters for updating a topic.
pub struct UpdateTopicParams<'a> {
  pub project_id: Uuid,
  pub topic_id: Uuid,
  pub title: Option<&'a str>,
  pub description: Option<&'a str>,
  pub topic_type: Option<&'a str>,
  pub topic_status: Option<&'a str>,
  pub priority: Option<&'a str>,
  pub assigned_to: Option<Uuid>,
  pub stage: Option<&'a str>,
  pub labels: Option<&'a serde_json::Value>,
  pub due_date: Option<chrono::NaiveDate>,
  pub index_number: Option<i32>,
}

/// List all topics for a project.
pub async fn list_topics(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<Vec<TopicRow>, sqlx::Error> {
  sqlx::query_as::<_, TopicRow>(
    "SELECT id, project_id, title, description, topic_type, topic_status,
            priority, assigned_to, stage, labels, due_date,
            creation_author, modified_author, index_number,
            created_at, updated_at
     FROM topics
     WHERE project_id = $1
     ORDER BY created_at DESC",
  )
  .bind(project_id)
  .fetch_all(pool)
  .await
}

/// Optional filters for listing topics.
pub struct TopicFilters {
  pub status: Option<String>,
  pub priority: Option<String>,
  pub assigned_to: Option<Uuid>,
}

/// List topics with optional filters.
pub async fn list_topics_filtered(
  pool: &PgPool,
  project_id: Uuid,
  filters: &TopicFilters,
) -> Result<Vec<TopicRow>, sqlx::Error> {
  let mut query = String::from(
    "SELECT id, project_id, title, description, topic_type, topic_status,
            priority, assigned_to, stage, labels, due_date,
            creation_author, modified_author, index_number,
            created_at, updated_at
     FROM topics
     WHERE project_id = $1",
  );

  let mut param_idx = 2u32;

  if filters.status.is_some() {
    query.push_str(&format!(" AND topic_status = ${param_idx}"));
    param_idx += 1;
  }
  if filters.priority.is_some() {
    query.push_str(&format!(" AND priority = ${param_idx}"));
    param_idx += 1;
  }
  if filters.assigned_to.is_some() {
    query.push_str(&format!(" AND assigned_to = ${param_idx}"));
    // param_idx not needed after this
  }

  query.push_str(" ORDER BY created_at DESC");

  let mut q = sqlx::query_as::<_, TopicRow>(&query).bind(project_id);

  if let Some(ref status) = filters.status {
    q = q.bind(status);
  }
  if let Some(ref priority) = filters.priority {
    q = q.bind(priority);
  }
  if let Some(assigned) = filters.assigned_to {
    q = q.bind(assigned);
  }

  q.fetch_all(pool).await
}

/// Get a single topic by ID within a project.
pub async fn get_topic(
  pool: &PgPool,
  project_id: Uuid,
  topic_id: Uuid,
) -> Result<Option<TopicRow>, sqlx::Error> {
  sqlx::query_as::<_, TopicRow>(
    "SELECT id, project_id, title, description, topic_type, topic_status,
            priority, assigned_to, stage, labels, due_date,
            creation_author, modified_author, index_number,
            created_at, updated_at
     FROM topics
     WHERE id = $1 AND project_id = $2",
  )
  .bind(topic_id)
  .bind(project_id)
  .fetch_optional(pool)
  .await
}

/// Get a single topic by ID (without project scope).
pub async fn get_topic_by_id(
  pool: &PgPool,
  topic_id: Uuid,
) -> Result<Option<TopicRow>, sqlx::Error> {
  sqlx::query_as::<_, TopicRow>(
    "SELECT id, project_id, title, description, topic_type, topic_status,
            priority, assigned_to, stage, labels, due_date,
            creation_author, modified_author, index_number,
            created_at, updated_at
     FROM topics
     WHERE id = $1",
  )
  .bind(topic_id)
  .fetch_optional(pool)
  .await
}

/// Insert a new topic.
pub async fn create_topic(
  pool: &PgPool,
  params: &CreateTopicParams<'_>,
) -> Result<TopicRow, sqlx::Error> {
  sqlx::query_as::<_, TopicRow>(
    "INSERT INTO topics (project_id, title, description, topic_type, topic_status,
                         priority, assigned_to, stage, labels, due_date, index_number,
                         creation_author)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
     RETURNING id, project_id, title, description, topic_type, topic_status,
               priority, assigned_to, stage, labels, due_date,
               creation_author, modified_author, index_number,
               created_at, updated_at",
  )
  .bind(params.project_id)
  .bind(params.title)
  .bind(params.description)
  .bind(params.topic_type)
  .bind(params.topic_status)
  .bind(params.priority)
  .bind(params.assigned_to)
  .bind(params.stage)
  .bind(params.labels)
  .bind(params.due_date)
  .bind(params.index_number)
  .bind(params.creation_author)
  .fetch_one(pool)
  .await
}

/// Update an existing topic. Returns None if not found.
pub async fn update_topic(
  pool: &PgPool,
  params: &UpdateTopicParams<'_>,
) -> Result<Option<TopicRow>, sqlx::Error> {
  sqlx::query_as::<_, TopicRow>(
    "UPDATE topics
     SET title = COALESCE($3, title),
         description = COALESCE($4, description),
         topic_type = COALESCE($5, topic_type),
         topic_status = COALESCE($6, topic_status),
         priority = COALESCE($7, priority),
         assigned_to = COALESCE($8, assigned_to),
         stage = COALESCE($9, stage),
         labels = COALESCE($10, labels),
         due_date = COALESCE($11, due_date),
         index_number = COALESCE($12, index_number),
         updated_at = now()
     WHERE id = $1 AND project_id = $2
     RETURNING id, project_id, title, description, topic_type, topic_status,
               priority, assigned_to, stage, labels, due_date,
               creation_author, modified_author, index_number,
               created_at, updated_at",
  )
  .bind(params.topic_id)
  .bind(params.project_id)
  .bind(params.title)
  .bind(params.description)
  .bind(params.topic_type)
  .bind(params.topic_status)
  .bind(params.priority)
  .bind(params.assigned_to)
  .bind(params.stage)
  .bind(params.labels)
  .bind(params.due_date)
  .bind(params.index_number)
  .fetch_optional(pool)
  .await
}

/// Delete a topic. Returns true if deleted.
pub async fn delete_topic(
  pool: &PgPool,
  project_id: Uuid,
  topic_id: Uuid,
) -> Result<bool, sqlx::Error> {
  let result = sqlx::query("DELETE FROM topics WHERE id = $1 AND project_id = $2")
    .bind(topic_id)
    .bind(project_id)
    .execute(pool)
    .await?;
  Ok(result.rows_affected() > 0)
}
