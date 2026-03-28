//! Shared application state passed to all Axum handlers.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::auth::oidc::OidcClient;
use crate::config::Config;
use crate::routes::auth_routes::PendingAuth;
use crate::storage::SnapshotStorage;
use crate::webdav::NextcloudClient;

/// Application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
  pub pool: PgPool,
  pub config: Arc<Config>,
  pub storage: SnapshotStorage,
  /// OIDC client (None when auth is disabled).
  pub oidc_client: Option<OidcClient>,
  /// Pending OIDC auth flows (csrf_token → PendingAuth).
  pub pending_auth: Arc<RwLock<HashMap<String, PendingAuth>>>,
  /// Nextcloud WebDAV client (None when cloud storage is not configured).
  pub nextcloud: Option<NextcloudClient>,
}

impl AppState {
  /// Create a new AppState with the given database pool, config, and optional OIDC client.
  pub fn new(pool: PgPool, config: Config, oidc_client: Option<OidcClient>) -> Self {
    let storage = SnapshotStorage::new(&config.storage_path);
    let nextcloud = if config.nextcloud_enabled {
      Some(NextcloudClient::new(
        config.nextcloud_url.as_deref().unwrap(),
        config.nextcloud_user.as_deref().unwrap(),
        config.nextcloud_pass.as_deref().unwrap(),
      ))
    } else {
      None
    };
    Self {
      pool,
      config: Arc::new(config),
      storage,
      oidc_client,
      pending_auth: Arc::new(RwLock::new(HashMap::new())),
      nextcloud,
    }
  }
}
