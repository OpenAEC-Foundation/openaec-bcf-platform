//! OpenAEC BCF Platform — main entry point.
//!
//! Starts the Axum web server with PostgreSQL connection pool,
//! runs database migrations, and serves the BCF v2.1 API.

mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod state;
mod storage;

use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::auth::oidc::OidcClient;
use crate::config::Config;
use crate::routes::api_router;
use crate::state::AppState;

const MAX_DB_CONNECTIONS: u32 = 10;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Load .env file if present
  dotenvy::dotenv().ok();

  // Initialize structured logging
  tracing_subscriber::fmt()
    .with_env_filter(
      EnvFilter::try_from_default_env().unwrap_or_else(|_| "bcf_server=info,tower_http=info".into()),
    )
    .init();

  // Load configuration
  let config = Config::from_env()?;
  tracing::info!("starting bcf-server on {}:{}", config.host, config.port);

  // Create database connection pool
  let pool = PgPoolOptions::new()
    .max_connections(MAX_DB_CONNECTIONS)
    .connect(&config.database_url)
    .await?;

  // Run migrations
  tracing::info!("running database migrations");
  sqlx::migrate!("../../migrations").run(&pool).await?;
  tracing::info!("migrations complete");

  // Initialize OIDC client if auth is enabled
  let oidc_client = if config.auth_enabled {
    tracing::info!("auth enabled — discovering OIDC provider");
    let client = OidcClient::discover(
      config.oidc_issuer_url.as_deref().unwrap(),
      config.oidc_client_id.as_deref().unwrap(),
      config.oidc_client_secret.as_deref().unwrap(),
      config.oidc_redirect_uri.as_deref().unwrap(),
    )
    .await?;
    tracing::info!("OIDC provider discovered successfully");
    Some(client)
  } else {
    tracing::info!("auth disabled — running in open access mode");
    None
  };

  // Build application
  let state = AppState::new(pool, config.clone(), oidc_client);

  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  // Serve frontend SPA if static dir exists
  let static_dir = std::path::Path::new("/app/static");
  let app = if static_dir.exists() {
    tracing::info!("serving frontend from /app/static");
    let serve_dir = ServeDir::new(static_dir)
      .not_found_service(ServeFile::new(static_dir.join("index.html")));
    api_router()
      .fallback_service(serve_dir)
      .layer(cors)
      .layer(TraceLayer::new_for_http())
      .with_state(state)
  } else {
    tracing::info!("no static dir found, API-only mode");
    api_router()
      .layer(cors)
      .layer(TraceLayer::new_for_http())
      .with_state(state)
  };

  // Start server
  let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
  let listener = tokio::net::TcpListener::bind(addr).await?;
  tracing::info!("listening on {addr}");
  axum::serve(listener, app).await?;

  Ok(())
}
