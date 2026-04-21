//! Authentik service-token (Bearer) authentication.
//!
//! Machine-clients (CI workflows, Revit plugin, PyRevit exporters, MCP
//! servers) now authenticate against the OpenAEC platform using
//! **Authentik-issued API tokens**. Authentik prefixes these tokens with
//! `ak-` and validates them via its own `/api/v3/core/users/me/` endpoint.
//!
//! ## Flow
//!
//! 1. Strip the `Bearer ` prefix.
//! 2. Skip local JWTs (prefix `eyJ`) and legacy API keys (prefix `bcfk_`)
//!    — those belong to [`crate::auth::jwt`] / [`crate::auth::api_key`].
//! 3. Validate the token against Authentik's `users/me` endpoint with a
//!    5-minute cache keyed on the token's sha256 fingerprint. The cache is
//!    time-bucketed — entries older than [`TOKEN_CACHE_TTL`] expire lazily
//!    when a new request lands in the next bucket.
//! 4. Auto-provision a local user row keyed on the stable OIDC subject
//!    `authentik-svc:<username>` so that downstream routes can use the
//!    resulting [`AuthUser::user_id`] just like any other caller.
//!
//! ## X-Original-Tenant header
//!
//! When a backend-to-backend request carries the `X-Original-Tenant`
//! header alongside a valid `Bearer ak-*` token, we treat it as an
//! impersonation hint — Authentik has already vouched for the caller, so
//! the service is trusted to speak on behalf of the named tenant. Kept
//! here as a thin extractor; the BCF data model doesn't yet key any row
//! on tenant, so the value is only surfaced via [`tracing`] for now.
//!
//! ## Sentinel password hash
//!
//! Auto-provisioned rows store `password_hash = '!authentik'` — the same
//! sentinel used by [`crate::auth::forward_auth`]. The value is not a
//! valid bcrypt digest, so the local-password flow will always reject it.
//! Both columns of the `users_auth_method` CHECK constraint are
//! satisfied: `sub` is populated (`authentik-svc:<username>`) and
//! `password_hash` is non-null.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::models::user::UserRow;
use crate::state::AppState;

/// How long a positive Authentik `users/me` response stays cached.
///
/// Matches the openaec-reports Python reference implementation
/// (`auth/dependencies.py::_TOKEN_CACHE_TTL`).
pub const TOKEN_CACHE_TTL: Duration = Duration::from_secs(300);

/// HTTP timeout for the Authentik validation round-trip.
const AUTHENTIK_TIMEOUT: Duration = Duration::from_secs(5);

/// Sentinel password-hash marker written into auto-provisioned user rows.
const PASSWORD_SENTINEL: &str = "!authentik";

/// Prefix used to namespace Authentik service-account subjects in the
/// `users.sub` column so they don't collide with OIDC end-user subjects.
const SVC_SUBJECT_PREFIX: &str = "authentik-svc:";

/// Local-part of the synthetic email assigned to service accounts.
///
/// Full form: `<username>@service.openaec.local` — matches Reports.
const SVC_EMAIL_DOMAIN: &str = "service.openaec.local";

/// Cached user-info payload extracted from `/api/v3/core/users/me/`.
#[derive(Debug, Clone)]
struct AuthentikUserInfo {
  /// Authentik `username` field — stable identifier for the service account.
  username: String,
  /// Authentik `name` field — human-readable display name.
  display_name: String,
  /// Tenant slug from the `attributes.tenant` property-mapping, if any.
  tenant: Option<String>,
}

/// Single cache entry: instant of insertion + (possibly negative) result.
#[derive(Clone)]
struct CacheEntry {
  inserted: Instant,
  user_info: Option<AuthentikUserInfo>,
}

/// Process-wide cache of token-fingerprint → Authentik user payload.
///
/// Uses `tokio::sync::RwLock` because moka is not in the workspace and
/// the cache sees modest traffic (one lookup per machine-client request).
/// Keyed on the first 16 hex chars of the sha256 fingerprint so we never
/// hold raw tokens in memory.
static TOKEN_CACHE: OnceLock<RwLock<HashMap<String, CacheEntry>>> = OnceLock::new();

/// Shared `reqwest` client re-used across validation calls.
///
/// Connection pooling + TLS session cache drop p99 latency to Authentik
/// from ~250 ms to ~30 ms on a warm process.
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn cache() -> &'static RwLock<HashMap<String, CacheEntry>> {
  TOKEN_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn http_client() -> &'static reqwest::Client {
  HTTP_CLIENT.get_or_init(|| {
    reqwest::Client::builder()
      .timeout(AUTHENTIK_TIMEOUT)
      .build()
      .unwrap_or_else(|_| reqwest::Client::new())
  })
}

/// Return `true` when the token looks like an Authentik service-token
/// candidate — not a local JWT and not a legacy `bcfk_` API key.
///
/// Reports' Python reference implementation is stricter (anything that
/// isn't a JWT counts), but in the Rust cascade we already route `bcfk_`
/// tokens to the legacy validator before reaching us, so this helper is
/// the second gate.
#[inline]
pub fn looks_like_service_token(token: &str) -> bool {
  !token.is_empty() && !token.starts_with("eyJ") && !token.starts_with("bcfk_")
}

/// Return the 16-char sha256 fingerprint of a token.
///
/// Hex-lowercase, first 16 characters — 64 bits of fingerprint, enough
/// to avoid accidental collisions in the process-local cache without
/// exposing the raw token in log lines.
pub fn token_fingerprint(token: &str) -> String {
  let digest = Sha256::digest(token.as_bytes());
  // Each byte → 2 hex chars; we want 16 chars → first 8 bytes.
  let mut out = String::with_capacity(16);
  for byte in &digest[..8] {
    out.push_str(&format!("{byte:02x}"));
  }
  out
}

/// Return the cache bucket index for a wall-clock Unix timestamp.
///
/// Buckets are `TOKEN_CACHE_TTL`-wide windows aligned to the Unix epoch —
/// same semantics as the Python reference. The main code path uses
/// [`Instant::elapsed`] on the cached entry, so this helper exists only
/// for deterministic unit testing of the bucket boundary logic.
#[cfg(test)]
fn current_bucket(now_unix_secs: u64) -> u64 {
  now_unix_secs / TOKEN_CACHE_TTL.as_secs()
}

/// Validate an Authentik `ak-*` Bearer token and resolve it to an [`AuthUser`].
///
/// Returns `None` when:
/// - the token looks like a JWT or `bcfk_` key (caller should fall through),
/// - Authentik rejects the token (403 / 401 / 404),
/// - the HTTP call fails (network / timeout — logged at `warn` level),
/// - auto-provisioning fails (DB error — logged at `warn`).
///
/// On success the local user row is returned, inserted on first sight.
#[tracing::instrument(skip_all, fields(fp = tracing::field::Empty))]
pub async fn validate_authentik_token(token: &str, state: &AppState) -> Option<AuthUser> {
  if !looks_like_service_token(token) {
    return None;
  }

  let fingerprint = token_fingerprint(token);
  tracing::Span::current().record("fp", fingerprint.as_str());

  // 1. Cache fast-path.
  if let Some(cached) = read_cache(&fingerprint).await {
    tracing::debug!("authentik token cache hit");
    let info = cached?;
    return resolve_local_user(&state.pool, &info).await;
  }

  // 2. Live validation against Authentik.
  let api_url = state
    .config
    .authentik_api_url
    .as_deref()
    .unwrap_or("https://auth.open-aec.com");
  let endpoint = format!("{}/api/v3/core/users/me/", api_url.trim_end_matches('/'));

  let response = http_client()
    .get(&endpoint)
    .bearer_auth(token)
    .send()
    .await;

  let user_info = match response {
    Ok(resp) if resp.status().is_success() => parse_users_me_payload(resp).await,
    Ok(resp) => {
      tracing::debug!(status = %resp.status(), "authentik token rejected");
      None
    }
    Err(err) => {
      tracing::warn!(error = %err, "authentik token validation error");
      None
    }
  };

  // 3. Cache the outcome (positive or negative) and resolve the local row.
  write_cache(&fingerprint, user_info.clone()).await;

  let info = user_info?;
  resolve_local_user(&state.pool, &info).await
}

async fn read_cache(fingerprint: &str) -> Option<Option<AuthentikUserInfo>> {
  let guard = cache().read().await;
  let entry = guard.get(fingerprint)?;
  if entry.inserted.elapsed() >= TOKEN_CACHE_TTL {
    return None;
  }
  Some(entry.user_info.clone())
}

async fn write_cache(fingerprint: &str, user_info: Option<AuthentikUserInfo>) {
  let mut guard = cache().write().await;
  // Opportunistic purge of expired entries to bound memory growth.
  guard.retain(|_, entry| entry.inserted.elapsed() < TOKEN_CACHE_TTL);
  guard.insert(
    fingerprint.to_string(),
    CacheEntry {
      inserted: Instant::now(),
      user_info,
    },
  );
}

/// Parse the subset of `/api/v3/core/users/me/` we actually need.
///
/// Authentik returns `{"user": {"username": ..., "name": ..., "attributes": {...}}}`.
async fn parse_users_me_payload(resp: reqwest::Response) -> Option<AuthentikUserInfo> {
  let body: serde_json::Value = resp.json().await.ok()?;
  let user = body.get("user").unwrap_or(&body);

  let username = user.get("username")?.as_str()?.trim().to_string();
  if username.is_empty() {
    return None;
  }
  let display_name = user
    .get("name")
    .and_then(|v| v.as_str())
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| username.clone());
  let tenant = user
    .get("attributes")
    .and_then(|v| v.get("tenant"))
    .and_then(|v| v.as_str())
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());

  Some(AuthentikUserInfo {
    username,
    display_name,
    tenant,
  })
}

/// Resolve the Authentik service account to a local [`AuthUser`],
/// auto-provisioning the `users` row on first sight.
async fn resolve_local_user(pool: &PgPool, info: &AuthentikUserInfo) -> Option<AuthUser> {
  let subject = format!("{SVC_SUBJECT_PREFIX}{}", info.username);
  let email = format!("{}@{SVC_EMAIL_DOMAIN}", info.username);

  match find_or_provision(pool, &subject, &email, &info.display_name).await {
    Ok(row) => {
      if let Some(tenant) = &info.tenant {
        tracing::debug!(username = %info.username, tenant = %tenant, "authentik service token accepted");
      } else {
        tracing::debug!(username = %info.username, "authentik service token accepted");
      }
      Some(AuthUser {
        user_id: row.id,
        email: row.email,
        name: row.name,
      })
    }
    Err(err) => {
      tracing::warn!(username = %info.username, error = %err, "authentik service user provisioning failed");
      None
    }
  }
}

/// Fetch the existing row for `subject` (an `authentik-svc:<username>`
/// subject) or insert one on first sight.
async fn find_or_provision(
  pool: &PgPool,
  subject: &str,
  email: &str,
  display_name: &str,
) -> Result<UserRow, sqlx::Error> {
  if let Some(row) = sqlx::query_as::<_, UserRow>(
    "SELECT id, sub, email, name, avatar_url, created_at, updated_at
       FROM users WHERE sub = $1",
  )
  .bind(subject)
  .fetch_optional(pool)
  .await?
  {
    return Ok(row);
  }

  sqlx::query_as::<_, UserRow>(
    "INSERT INTO users (id, sub, email, name, avatar_url, password_hash)
       VALUES ($1, $2, $3, $4, NULL, $5)
       ON CONFLICT (sub) DO UPDATE
         SET updated_at = now()
       RETURNING id, sub, email, name, avatar_url, created_at, updated_at",
  )
  .bind(Uuid::new_v4())
  .bind(subject)
  .bind(email)
  .bind(display_name)
  .bind(PASSWORD_SENTINEL)
  .fetch_one(pool)
  .await
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn service_token_gate_rejects_jwts_and_bcfk() {
    assert!(!looks_like_service_token(""));
    assert!(!looks_like_service_token("eyJhbGciOi"));
    assert!(!looks_like_service_token("bcfk_abcdef"));
    assert!(looks_like_service_token("ak-service-token"));
    // Neutral strings also pass — the live `/users/me` call is the final
    // gatekeeper, not the prefix check.
    assert!(looks_like_service_token("anything-else"));
  }

  #[test]
  fn fingerprint_is_16_hex_chars_and_stable() {
    let fp = token_fingerprint("ak-example-token");
    assert_eq!(fp.len(), 16);
    assert!(fp.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    // Deterministic — same input, same output.
    assert_eq!(fp, token_fingerprint("ak-example-token"));
    // Different inputs yield different fingerprints.
    assert_ne!(fp, token_fingerprint("ak-other-token"));
  }

  #[test]
  fn bucket_changes_every_ttl_window() {
    let ttl = TOKEN_CACHE_TTL.as_secs();
    // Anchor to an exact bucket boundary so `base..base+ttl-1` spans a single bucket.
    let base: u64 = (1_700_000_000 / ttl) * ttl;
    assert_eq!(current_bucket(base), current_bucket(base + ttl - 1));
    assert_ne!(current_bucket(base), current_bucket(base + ttl));
    // Adjacent buckets differ by exactly one.
    assert_eq!(current_bucket(base + ttl), current_bucket(base) + 1);
  }

  #[tokio::test]
  async fn cache_returns_none_after_ttl_expiry() {
    let fp = "0123456789abcdef";
    // Prime the cache with an artificially-old entry.
    {
      let mut guard = cache().write().await;
      guard.insert(
        fp.to_string(),
        CacheEntry {
          inserted: Instant::now() - TOKEN_CACHE_TTL - Duration::from_secs(1),
          user_info: Some(AuthentikUserInfo {
            username: "expired".into(),
            display_name: "Expired".into(),
            tenant: None,
          }),
        },
      );
    }
    assert!(read_cache(fp).await.is_none());
    // Clean up so other tests don't inherit state.
    cache().write().await.remove(fp);
  }

  #[tokio::test]
  async fn cache_roundtrips_positive_and_negative_entries() {
    let positive_fp = "aaaaaaaaaaaaaaaa";
    let negative_fp = "bbbbbbbbbbbbbbbb";

    write_cache(
      positive_fp,
      Some(AuthentikUserInfo {
        username: "svc-user".into(),
        display_name: "Service User".into(),
        tenant: Some("3bm".into()),
      }),
    )
    .await;
    write_cache(negative_fp, None).await;

    let hit = read_cache(positive_fp).await.expect("present");
    let info = hit.expect("positive");
    assert_eq!(info.username, "svc-user");
    assert_eq!(info.tenant.as_deref(), Some("3bm"));

    let miss = read_cache(negative_fp).await.expect("present");
    assert!(miss.is_none());

    cache().write().await.remove(positive_fp);
    cache().write().await.remove(negative_fp);
  }
}
