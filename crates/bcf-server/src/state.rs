//! Shared application state passed to all Axum handlers.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::auth::oidc::OidcClient;
use crate::config::Config;
use crate::routes::auth_routes::PendingAuth;
use crate::storage::SnapshotStorage;
use crate::webdav::{CloudClient, TenantConfig, TenantsRegistry};

/// Tool slug used for cloud storage directory mapping.
const TOOL_SLUG: &str = "bcf-platform";

/// Application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub storage: SnapshotStorage,
    /// OIDC client (None when auth is disabled).
    pub oidc_client: Option<OidcClient>,
    /// Pending OIDC auth flows (csrf_token -> PendingAuth).
    pub pending_auth: Arc<RwLock<HashMap<String, PendingAuth>>>,
    /// Cloud storage client (None when cloud storage is not configured).
    /// Wrapped in Arc because CloudClient does not implement Clone.
    pub cloud: Option<Arc<CloudClient>>,
}

impl AppState {
    /// Create a new AppState with the given database pool, config, and optional OIDC client.
    pub fn new(pool: PgPool, config: Config, oidc_client: Option<OidcClient>) -> Self {
        let storage = SnapshotStorage::new(&config.storage_path);
        let cloud = build_cloud_client(&config).map(Arc::new);
        Self {
            pool,
            config: Arc::new(config),
            storage,
            oidc_client,
            pending_auth: Arc::new(RwLock::new(HashMap::new())),
            cloud,
        }
    }
}

/// Build the cloud client from configuration.
///
/// Prefers multi-tenant mode (TENANTS_CONFIG) over legacy single-tenant
/// (NEXTCLOUD_URL + NEXTCLOUD_SERVICE_USER + NEXTCLOUD_SERVICE_PASS).
fn build_cloud_client(config: &Config) -> Option<CloudClient> {
    // Try multi-tenant registry first
    if let Ok(registry) = TenantsRegistry::load_from_env() {
        if registry.is_configured() {
            // Use default_tenant or first available tenant
            let slug = config
                .default_tenant
                .as_deref()
                .or_else(|| registry.slugs().into_iter().next());

            if let Some(slug) = slug {
                if let Some(tenant) = registry.get(slug) {
                    tracing::info!(
                        tenant = %slug,
                        "cloud storage enabled via multi-tenant registry"
                    );
                    return Some(CloudClient::new(tenant, TOOL_SLUG));
                }
            }
        }
    }

    // Fallback: legacy single-tenant env vars
    if let (Some(url), Some(user), Some(pass)) = (
        config.nextcloud_url.as_deref(),
        config.nextcloud_user.as_deref(),
        config.nextcloud_pass.as_deref(),
    ) {
        tracing::info!(url = %url, "cloud storage enabled via legacy env vars");
        let tenant = TenantConfig {
            slug: "default".to_string(),
            name: "Default".to_string(),
            nextcloud_url: url.to_string(),
            nextcloud_domain: String::new(),
            service_user: user.to_string(),
            service_pass: pass.to_string(),
            group_folder_id: 1,
            volume_mount: String::new(),
        };
        return Some(CloudClient::new(&tenant, TOOL_SLUG));
    }

    tracing::info!("cloud storage disabled (no tenant config or legacy env vars)");
    None
}
