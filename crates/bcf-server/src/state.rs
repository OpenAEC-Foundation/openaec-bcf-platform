//! Shared application state passed to all Axum handlers.

use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;
use crate::storage::SnapshotStorage;

/// Application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
  pub pool: PgPool,
  pub config: Arc<Config>,
  pub storage: SnapshotStorage,
}

impl AppState {
  /// Create a new AppState with the given database pool and config.
  pub fn new(pool: PgPool, config: Config) -> Self {
    let storage = SnapshotStorage::new(&config.storage_path);
    Self {
      pool,
      config: Arc::new(config),
      storage,
    }
  }
}
