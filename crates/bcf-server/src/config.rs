//! Application configuration loaded from environment variables.

/// Server and database configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
  pub database_url: String,
  pub host: String,
  pub port: u16,
  pub storage_path: String,
}

impl Config {
  /// Load configuration from environment variables.
  ///
  /// Required: `DATABASE_URL`
  /// Optional: `HOST` (default 0.0.0.0), `PORT` (default 3000),
  ///           `STORAGE_PATH` (default ./data/snapshots)
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

    Ok(Self {
      database_url,
      host,
      port,
      storage_path,
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
