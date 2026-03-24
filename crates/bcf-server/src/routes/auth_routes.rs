//! Authentication route handlers (OIDC login flow).
//!
//! GET  /auth/login    — Redirect to OIDC provider
//! GET  /auth/callback — Exchange code, provision user, issue JWT
//! GET  /auth/me       — Return current user info

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::jwt::create_session_token;
use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::user::UserResponse;
use crate::state::AppState;

/// Auth routes.
pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/auth/login", get(login))
    .route("/auth/callback", get(callback))
    .route("/auth/me", get(me))
}

/// GET /auth/login — Start the OIDC login flow.
async fn login(State(state): State<AppState>) -> Result<Response, AppError> {
  let oidc = state
    .oidc_client
    .as_ref()
    .ok_or_else(|| AppError::Internal("OIDC not configured".to_string()))?;

  let (auth_url, csrf_state, nonce, pkce_verifier) = oidc.authorize_url();

  // Store PKCE verifier and nonce for the callback
  state.pending_auth.write().await.insert(
    csrf_state,
    PendingAuth {
      pkce_verifier,
      nonce,
    },
  );

  Ok(Redirect::temporary(&auth_url).into_response())
}

/// Query parameters for the OIDC callback.
#[derive(Debug, Deserialize)]
struct CallbackParams {
  code: String,
  state: String,
}

/// GET /auth/callback — Exchange authorization code for tokens.
async fn callback(
  State(state): State<AppState>,
  Query(params): Query<CallbackParams>,
) -> Result<Response, AppError> {
  let oidc = state
    .oidc_client
    .as_ref()
    .ok_or_else(|| AppError::Internal("OIDC not configured".to_string()))?;

  // Retrieve and consume the pending auth state
  let pending = state
    .pending_auth
    .write()
    .await
    .remove(&params.state)
    .ok_or(AppError::BadRequest("invalid or expired state parameter".to_string()))?;

  // Exchange code for tokens and extract claims
  let claims = oidc
    .exchange_code(&params.code, &pending.pkce_verifier, &pending.nonce)
    .await
    .map_err(|e| AppError::Internal(format!("OIDC token exchange failed: {e}")))?;

  // Upsert user in database (JIT provisioning)
  let user = db::users::upsert_from_oidc(&state.pool, &claims)
    .await
    .map_err(|e| AppError::Internal(format!("user provisioning failed: {e}")))?;

  tracing::info!(user_id = %user.id, email = %user.email, "user authenticated via OIDC");

  // Create session JWT
  let token = create_session_token(
    user.id,
    &user.email,
    &user.name,
    &state.config.jwt_secret,
  )
  .map_err(|e| AppError::Internal(format!("failed to create session token: {e}")))?;

  // Redirect to frontend with token
  let redirect_url = format!("{}?token={}", state.config.frontend_url, token);
  Ok(Redirect::temporary(&redirect_url).into_response())
}

/// GET /auth/me — Return the current authenticated user.
async fn me(auth: AuthUser) -> AppResult<Json<UserResponse>> {
  Ok(Json(UserResponse {
    user_id: auth.user_id,
    email: auth.email.clone(),
    name: auth.name.clone(),
    avatar_url: None,
  }))
}

/// Pending OIDC auth state stored between login redirect and callback.
pub struct PendingAuth {
  pub pkce_verifier: String,
  pub nonce: String,
}
