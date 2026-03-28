//! User management routes.
//!
//! GET  /api/v1/users          — search users (for member picker)
//! POST /api/v1/users          — create local platform user
//! POST /auth/local/login      — login with email + password

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::auth::jwt;
use crate::error::{AppError, AppResult};
use crate::models::user::UserResponse;
use crate::state::AppState;

/// User routes.
pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/users", get(search_users).post(create_local_user))
}

/// Auth routes for local login.
pub fn local_auth_routes() -> Router<AppState> {
  Router::new().route("/auth/local/login", post(local_login))
}

/// Query parameters for user search.
#[derive(Debug, Deserialize)]
struct UserSearchParams {
  q: Option<String>,
}

/// GET /api/v1/users?q=search — Search users by name or email.
async fn search_users(
  State(state): State<AppState>,
  Query(params): Query<UserSearchParams>,
) -> AppResult<Json<Vec<UserResponse>>> {
  let rows = if let Some(ref q) = params.q {
    let pattern = format!("%{q}%");
    sqlx::query_as::<_, crate::models::user::UserRow>(
      "SELECT id, sub, email, name, avatar_url, created_at, updated_at
       FROM users
       WHERE name ILIKE $1 OR email ILIKE $1
       ORDER BY name
       LIMIT 50",
    )
    .bind(&pattern)
    .fetch_all(&state.pool)
    .await?
  } else {
    sqlx::query_as::<_, crate::models::user::UserRow>(
      "SELECT id, sub, email, name, avatar_url, created_at, updated_at
       FROM users
       ORDER BY name
       LIMIT 50",
    )
    .fetch_all(&state.pool)
    .await?
  };

  let users: Vec<UserResponse> = rows.into_iter().map(Into::into).collect();
  Ok(Json(users))
}

/// Request body for creating a local user.
#[derive(Debug, Deserialize)]
struct CreateLocalUserRequest {
  email: String,
  name: String,
  password: String,
}

/// POST /api/v1/users — Create a local platform user.
async fn create_local_user(
  State(state): State<AppState>,
  Json(body): Json<CreateLocalUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
  if body.email.trim().is_empty() || body.name.trim().is_empty() {
    return Err(AppError::BadRequest("email and name are required".to_string()));
  }
  if body.password.len() < 8 {
    return Err(AppError::BadRequest(
      "password must be at least 8 characters".to_string(),
    ));
  }

  // Hash password
  let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
    .map_err(|e| AppError::Internal(format!("hash error: {e}")))?;

  let row = sqlx::query_as::<_, crate::models::user::UserRow>(
    "INSERT INTO users (email, name, password_hash)
     VALUES ($1, $2, $3)
     RETURNING id, sub, email, name, avatar_url, created_at, updated_at",
  )
  .bind(&body.email)
  .bind(&body.name)
  .bind(&password_hash)
  .fetch_one(&state.pool)
  .await
  .map_err(|e| {
    if let sqlx::Error::Database(ref db_err) = e {
      if db_err.constraint() == Some("users_auth_method") {
        return AppError::BadRequest("email already exists".to_string());
      }
    }
    AppError::Database(e)
  })?;

  Ok((StatusCode::CREATED, Json(row.into())))
}

/// Login response with JWT token.
#[derive(Serialize)]
struct LoginResponse {
  token: String,
  user: UserResponse,
}

/// Request body for local login.
#[derive(Deserialize)]
struct LocalLoginRequest {
  email: String,
  password: String,
}

/// POST /auth/local/login — Authenticate with email + password.
async fn local_login(
  State(state): State<AppState>,
  Json(body): Json<LocalLoginRequest>,
) -> AppResult<Json<LoginResponse>> {
  // Find user by email
  let row = sqlx::query_as::<_, crate::models::user::UserRow>(
    "SELECT id, sub, email, name, avatar_url, created_at, updated_at
     FROM users WHERE email = $1",
  )
  .bind(&body.email)
  .fetch_optional(&state.pool)
  .await?
  .ok_or(AppError::Unauthorized)?;

  // Get password hash
  let hash_row: Option<(Option<String>,)> = sqlx::query_as(
    "SELECT password_hash FROM users WHERE id = $1",
  )
  .bind(row.id)
  .fetch_optional(&state.pool)
  .await?;

  let password_hash = hash_row
    .and_then(|r| r.0)
    .ok_or(AppError::Unauthorized)?;

  // Verify password
  let valid = bcrypt::verify(&body.password, &password_hash)
    .map_err(|_| AppError::Unauthorized)?;
  if !valid {
    return Err(AppError::Unauthorized);
  }

  // Issue JWT
  let token = jwt::create_session_token(row.id, &row.email, &row.name, &state.config.jwt_secret)
    .map_err(|e| AppError::Internal(format!("jwt error: {e}")))?;

  Ok(Json(LoginResponse {
    token,
    user: row.into(),
  }))
}
