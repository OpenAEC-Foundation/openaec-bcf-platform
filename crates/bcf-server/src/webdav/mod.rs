//! Cloud storage integration via the shared `openaec-cloud` crate.
//!
//! Re-exports the [`openaec_cloud::CloudClient`] as the primary interface
//! and provides error conversion to [`AppError`].

pub use openaec_cloud::{
    CloudClient, CloudError, TenantsRegistry, TenantConfig,
};

use crate::error::AppError;

/// Convert [`CloudError`] to [`AppError`] for use in Axum handlers.
impl From<CloudError> for AppError {
    fn from(err: CloudError) -> Self {
        match err {
            CloudError::NotFound(msg) => AppError::NotFound(msg),
            CloudError::Nextcloud(msg) => AppError::Internal(format!("cloud storage: {msg}")),
            CloudError::Io(err) => AppError::Internal(format!("cloud I/O: {err}")),
        }
    }
}
