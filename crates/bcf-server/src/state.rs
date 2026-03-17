//! Shared application state passed to all Axum handlers.

use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;

/// Application state shared across all request handlers.
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
  pub pool: PgPool,
  pub config: Arc<Config>,
}

impl AppState {
  /// Create a new AppState with the given database pool and config.
  pub fn new(pool: PgPool, config: Config) -> Self {
    Self {
      pool,
      config: Arc::new(config),
    }
  }
}
