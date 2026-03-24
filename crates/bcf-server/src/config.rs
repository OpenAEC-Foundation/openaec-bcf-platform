//! Application configuration loaded from environment variables.

/// Server and database configuration.
#[derive(Debug, Clone)]
pub struct Config {
  pub database_url: String,
  pub host: String,
  pub port: u16,
  pub storage_path: String,
  /// Enable authentication middleware. When false, all routes are open.
  pub auth_enabled: bool,
  /// OIDC issuer URL (e.g. `https://auth.open-aec.com/application/o/bcf-platform/`).
  pub oidc_issuer_url: Option<String>,
  /// OIDC client ID.
  pub oidc_client_id: Option<String>,
  /// OIDC client secret.
  pub oidc_client_secret: Option<String>,
  /// OIDC redirect URI (e.g. `https://bcf.open-aec.com/auth/callback`).
  pub oidc_redirect_uri: Option<String>,
  /// Secret used to sign session JWTs. Must be at least 32 characters.
  pub jwt_secret: String,
  /// Frontend URL to redirect to after OIDC callback.
  pub frontend_url: String,
}

impl Config {
  /// Load configuration from environment variables.
  ///
  /// Required: `DATABASE_URL`
  /// Optional: `HOST` (default 0.0.0.0), `PORT` (default 3000),
  ///           `STORAGE_PATH` (default ./data/snapshots),
  ///           `AUTH_ENABLED` (default false),
  ///           `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`,
  ///           `OIDC_REDIRECT_URI`, `JWT_SECRET`, `FRONTEND_URL`
  pub fn from_env() -> Result<Self, ConfigError> {
    let database_url = std::env::var("DATABASE_URL")
      .map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port = std::env::var("PORT")
      .unwrap_or_else(|_| "3000".to_string())
      .parse::<u16>()
      .map_err(|_| ConfigError::Invalid("PORT", "must be a valid u16"))?;

    let storage_path = std::env::var("STORAGE_PATH")
      .unwrap_or_else(|_| "./data/snapshots".to_string());

    let auth_enabled = std::env::var("AUTH_ENABLED")
      .unwrap_or_else(|_| "false".to_string())
      .parse::<bool>()
      .unwrap_or(false);

    let oidc_issuer_url = std::env::var("OIDC_ISSUER_URL").ok();
    let oidc_client_id = std::env::var("OIDC_CLIENT_ID").ok();
    let oidc_client_secret = std::env::var("OIDC_CLIENT_SECRET").ok();
    let oidc_redirect_uri = std::env::var("OIDC_REDIRECT_URI").ok();

    let jwt_secret = std::env::var("JWT_SECRET")
      .unwrap_or_else(|_| "dev-secret-change-me-in-production-32chars!".to_string());

    let frontend_url = std::env::var("FRONTEND_URL")
      .unwrap_or_else(|_| "http://localhost:5173".to_string());

    // Validate: if auth is enabled, OIDC settings are required
    if auth_enabled {
      if oidc_issuer_url.is_none() {
        return Err(ConfigError::Missing("OIDC_ISSUER_URL (required when AUTH_ENABLED=true)"));
      }
      if oidc_client_id.is_none() {
        return Err(ConfigError::Missing("OIDC_CLIENT_ID (required when AUTH_ENABLED=true)"));
      }
      if oidc_client_secret.is_none() {
        return Err(ConfigError::Missing("OIDC_CLIENT_SECRET (required when AUTH_ENABLED=true)"));
      }
      if oidc_redirect_uri.is_none() {
        return Err(ConfigError::Missing("OIDC_REDIRECT_URI (required when AUTH_ENABLED=true)"));
      }
    }

    Ok(Self {
      database_url,
      host,
      port,
      storage_path,
      auth_enabled,
      oidc_issuer_url,
      oidc_client_id,
      oidc_client_secret,
      oidc_redirect_uri,
      jwt_secret,
      frontend_url,
    })
  }
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
  #[error("missing required environment variable: {0}")]
  Missing(&'static str),
  #[error("invalid value for {0}: {1}")]
  Invalid(&'static str, &'static str),
}
