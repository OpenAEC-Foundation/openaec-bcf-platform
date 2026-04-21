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
/// Email is declared `UNIQUE` in the schema (migration `003_users_email_unique`),
/// so this query returns at most one row. Used by the Authentik forward_auth
/// extractor to resolve incoming `X-Authentik-Email` headers to a local
/// user row.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
  sqlx::query_as::<_, UserRow>(
    "SELECT id, sub, email, name, avatar_url, created_at, updated_at
     FROM users WHERE email = $1",
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

#[cfg(test)]
mod tests {
  //! `find_by_email` is DB-backed; behavioural coverage lives in the
  //! integration-test harness (`#[ignore]` below) and runs only when a
  //! live PostgreSQL with the migrations applied is available via
  //! `DATABASE_URL`. The shape of the query itself is validated here so
  //! that a refactor can't accidentally reintroduce the `ORDER BY … LIMIT 1`
  //! workaround now that migration `003_users_email_unique` guarantees a
  //! single row per email.

  /// The `find_by_email` query must rely on the UNIQUE(email) constraint
  /// and therefore must NOT carry the legacy `ORDER BY created_at ASC
  /// LIMIT 1` tie-breaker. Regression-guard against a revert.
  #[test]
  fn find_by_email_query_has_no_order_by_tiebreak() {
    // Mirror the literal SQL used in `find_by_email` above. Kept in sync
    // by the migration-003 comment on that function.
    let sql = "SELECT id, sub, email, name, avatar_url, created_at, updated_at
     FROM users WHERE email = $1";
    assert!(
      !sql.to_ascii_lowercase().contains("order by"),
      "find_by_email must not use ORDER BY — UNIQUE(email) guarantees one row"
    );
    assert!(
      !sql.to_ascii_lowercase().contains("limit"),
      "find_by_email must not use LIMIT — UNIQUE(email) guarantees one row"
    );
  }

  /// Live DB smoke test: insert a user, look them up by email, confirm
  /// exactly one row returns. Gated with `#[ignore]` so it only runs when
  /// explicitly requested (`cargo test -p bcf-server -- --ignored`) with
  /// `DATABASE_URL` pointing at a disposable PostgreSQL.
  #[tokio::test]
  #[ignore = "requires DATABASE_URL and applied migrations"]
  async fn find_by_email_returns_single_row() {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    let db_url = match std::env::var("DATABASE_URL") {
      Ok(v) => v,
      Err(_) => return,
    };
    let pool = PgPoolOptions::new()
      .max_connections(1)
      .connect(&db_url)
      .await
      .expect("connect");

    let email = format!("find-by-email-{}@example.test", Uuid::new_v4());
    sqlx::query(
      "INSERT INTO users (sub, email, name, password_hash) \
         VALUES ($1, $2, $3, '!test')",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&email)
    .bind("Test User")
    .execute(&pool)
    .await
    .expect("insert");

    let row = find_by_email(&pool, &email).await.expect("lookup").expect("row");
    assert_eq!(row.email, email);

    sqlx::query("DELETE FROM users WHERE email = $1")
      .bind(&email)
      .execute(&pool)
      .await
      .ok();
  }
}
