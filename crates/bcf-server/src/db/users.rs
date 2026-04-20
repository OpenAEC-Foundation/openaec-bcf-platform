//! User database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{OidcUserClaims, UserRow};

/// Find a user by OIDC subject claim.
#[allow(dead_code)]
pub async fn find_by_sub(pool: &PgPool, sub: &str) -> Result<Option<UserRow>, sqlx::Error> {
  sqlx::query_as::<_, UserRow>(
    "SELECT id, sub, email, name, avatar_url, created_at, updated_at
     FROM users WHERE sub = $1",
  )
  .bind(sub)
  .fetch_optional(pool)
  .await
}

/// Find a user by ID.
pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
  sqlx::query_as::<_, UserRow>(
    "SELECT id, sub, email, name, avatar_url, created_at, updated_at
     FROM users WHERE id = $1",
  )
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

/// Find a user by email address.
///
/// Email is not declared `UNIQUE` in the schema, but in practice we treat
/// the first match as authoritative. Used by the Authentik forward_auth
/// extractor to resolve incoming `X-Authentik-Email` headers to a local
/// user row.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
  sqlx::query_as::<_, UserRow>(
    "SELECT id, sub, email, name, avatar_url, created_at, updated_at
     FROM users WHERE email = $1
     ORDER BY created_at ASC
     LIMIT 1",
  )
  .bind(email)
  .fetch_optional(pool)
  .await
}

/// Upsert a user from OIDC claims (insert or update on conflict).
pub async fn upsert_from_oidc(
  pool: &PgPool,
  claims: &OidcUserClaims,
) -> Result<UserRow, sqlx::Error> {
  sqlx::query_as::<_, UserRow>(
    "INSERT INTO users (sub, email, name, avatar_url)
     VALUES ($1, $2, $3, $4)
     ON CONFLICT (sub) DO UPDATE
       SET email = EXCLUDED.email,
           name = EXCLUDED.name,
           avatar_url = EXCLUDED.avatar_url,
           updated_at = now()
     RETURNING id, sub, email, name, avatar_url, created_at, updated_at",
  )
  .bind(&claims.sub)
  .bind(&claims.email)
  .bind(&claims.name)
  .bind(&claims.avatar_url)
  .fetch_one(pool)
  .await
}
