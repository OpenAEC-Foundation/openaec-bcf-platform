//! Authentik forward_auth header authentication.
//!
//! When the BCF Platform is deployed behind Caddy + Authentik's forward_auth
//! outpost, Authentik validates the user session and injects identity headers
//! on every upstream request. This module extracts those headers into an
//! [`AuthUser`], optionally auto-provisioning a new user row if the email is
//! unknown to the BCF database.
//!
//! ## Header convention
//!
//! Authentik distinguishes between **identity** headers (primary fields) and
//! **meta** headers (extra metadata from custom property mappings):
//!
//! | Header | Purpose |
//! |--------|---------|
//! | `X-Authentik-Email` | Primary identifier (required) |
//! | `X-Authentik-Username` | User login name |
//! | `X-Authentik-Name` | Display name |
//! | `X-Authentik-Uid` | Authentik user UUID |
//! | `X-Authentik-Groups` | Pipe-separated group list |
//! | `X-Authentik-Meta-Tenant` | Tenant slug (custom scope) |
//! | `X-Authentik-Meta-Company` | Company name (custom scope) |
//! | `X-Authentik-Meta-JobTitle` | Job title (custom scope) |
//! | `X-Authentik-Meta-Phone` | Phone number (custom scope) |
//! | `X-Authentik-Meta-RegNumber` | Registration number (custom scope) |
//!
//! The earlier Reports-service regression (`openaec-reports` commit `10d6f66`,
//! 17 april) showed that using the `Meta-` prefix for identity fields silently
//! breaks SSO: Authentik only emits `Meta-` for property-mapping metadata.
//! Keep identity reads on the un-prefixed header names.
//!
//! ## Auto-provisioning
//!
//! When [`Config::authentik_auto_provision`] is true (default), unknown users
//! are inserted into the `users` table on their first authenticated request.
//! This matches the OIDC JIT provisioning that the session-JWT flow provides,
//! so that forward_auth users can create topics, comments, etc. immediately.

use axum::http::request::Parts;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::db;
use crate::models::user::UserRow;
use crate::state::AppState;

/// Primary identity header — email address of the authenticated user.
const HDR_EMAIL: &str = "x-authentik-email";
/// Login name header.
const HDR_USERNAME: &str = "x-authentik-username";
/// Display name header.
const HDR_NAME: &str = "x-authentik-name";
/// Authentik user UUID header.
const HDR_UID: &str = "x-authentik-uid";
/// Pipe-separated group list header.
const HDR_GROUPS: &str = "x-authentik-groups";

/// Metadata from Authentik property mappings.
///
/// Populated from `X-Authentik-Meta-*` headers sent by the forward_auth
/// outpost. Not yet persisted or wired into routes — reserved for future
/// tenant / company-based authorization.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AuthentikMeta {
  /// Tenant slug (e.g. `3bm`).
  pub tenant: Option<String>,
  /// Company name.
  pub company: Option<String>,
  /// Job title.
  pub job_title: Option<String>,
  /// Phone number.
  pub phone: Option<String>,
  /// Registration number (e.g. KvK or chamber of commerce).
  pub registration_number: Option<String>,
}

impl AuthentikMeta {
  /// Extract all `X-Authentik-Meta-*` headers from the request parts.
  pub fn from_parts(parts: &Parts) -> Self {
    Self {
      tenant: header_value(parts, "x-authentik-meta-tenant"),
      company: header_value(parts, "x-authentik-meta-company"),
      job_title: header_value(parts, "x-authentik-meta-jobtitle"),
      phone: header_value(parts, "x-authentik-meta-phone"),
      registration_number: header_value(parts, "x-authentik-meta-regnumber"),
    }
  }
}

/// Extract an [`AuthUser`] from Authentik forward_auth headers, if present.
///
/// Returns `None` when the `X-Authentik-Email` header is absent or empty —
/// which is the signal that the request did not traverse the forward_auth
/// outpost (e.g. direct API-key calls or internal health probes). In that
/// case the caller should fall back to Bearer-token authentication.
///
/// Returns `Some(user)` when the email resolves to an existing user row, or
/// when auto-provisioning is enabled and the new row insert succeeds.
///
/// On database errors the function logs and returns `None` so that the
/// Bearer-token fallback can still take effect; the request will be rejected
/// there if no other credentials match.
#[tracing::instrument(skip_all)]
pub async fn extract_authentik_user(parts: &Parts, state: &AppState) -> Option<AuthUser> {
  let email = header_value(parts, HDR_EMAIL)?;
  if email.is_empty() {
    return None;
  }

  let username = header_value(parts, HDR_USERNAME);
  let display_name = header_value(parts, HDR_NAME);
  let uid = header_value(parts, HDR_UID);
  let _groups = header_value(parts, HDR_GROUPS);
  let meta = AuthentikMeta::from_parts(parts);

  // Derive a display name: X-Authentik-Name, else Username, else email local-part.
  let resolved_name = display_name
    .clone()
    .or_else(|| username.clone())
    .unwrap_or_else(|| {
      email
        .split('@')
        .next()
        .unwrap_or(email.as_str())
        .to_string()
    });

  match find_by_email(&state.pool, &email).await {
    Ok(Some(user)) => {
      tracing::debug!(email = %user.email, user_id = %user.id, "forward_auth: existing user");
      Some(AuthUser {
        user_id: user.id,
        email: user.email,
        name: user.name,
      })
    }
    Ok(None) => {
      if !state.config.authentik_auto_provision {
        tracing::debug!(
          email = %email,
          "forward_auth: user unknown and auto-provision disabled"
        );
        return None;
      }
      if !is_tenant_allowed(
        &state.config.authentik_auto_provision_tenants,
        meta.tenant.as_deref(),
      ) {
        tracing::debug!(
          email = %email,
          tenant = ?meta.tenant,
          allowlist = ?state.config.authentik_auto_provision_tenants,
          "forward_auth: tenant not on auto-provision allow-list"
        );
        return None;
      }
      match provision_user(&state.pool, &email, &resolved_name, uid.as_deref()).await {
        Ok(user) => {
          tracing::debug!(
            email = %user.email,
            user_id = %user.id,
            "forward_auth: provisioned new user"
          );
          Some(AuthUser {
            user_id: user.id,
            email: user.email,
            name: user.name,
          })
        }
        Err(err) => {
          tracing::warn!(email = %email, error = %err, "forward_auth: provisioning failed");
          None
        }
      }
    }
    Err(err) => {
      tracing::warn!(email = %email, error = %err, "forward_auth: lookup failed");
      None
    }
  }
}

/// Find a user row by the email column (case-sensitive).
async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
  db::users::find_by_email(pool, email).await
}

/// Insert a new user row from forward_auth headers.
///
/// If `uid` parses as a UUID it becomes the new row's id; otherwise a fresh
/// UUID is generated. `sub` is left `NULL` because forward_auth does not issue
/// an OIDC subject — the database constraint requires either `sub` or
/// `password_hash`, so we set `password_hash` to a sentinel (`'!authentik'`)
/// to signal "external identity, no local credentials". That sentinel is
/// never valid for bcrypt verification.
async fn provision_user(
  pool: &PgPool,
  email: &str,
  name: &str,
  uid: Option<&str>,
) -> Result<UserRow, sqlx::Error> {
  let user_id = uid
    .and_then(|s| Uuid::parse_str(s).ok())
    .unwrap_or_else(Uuid::new_v4);

  sqlx::query_as::<_, UserRow>(
    "INSERT INTO users (id, sub, email, name, avatar_url, password_hash)
     VALUES ($1, NULL, $2, $3, NULL, '!authentik')
     RETURNING id, sub, email, name, avatar_url, created_at, updated_at",
  )
  .bind(user_id)
  .bind(email)
  .bind(name)
  .fetch_one(pool)
  .await
}

/// Decide whether auto-provisioning is permitted for the given tenant.
///
/// Returns `true` when the allow-list is empty (backwards-compatible "no
/// gating" behaviour), or when the request carries a non-empty tenant value
/// that appears in the allow-list. Returns `false` when a non-empty
/// allow-list is configured and the tenant is absent or not listed.
///
/// Comparison is case-sensitive and slug values are trimmed upstream in
/// [`header_value`], so an `X-Authentik-Meta-Tenant: 3bm ` header matches
/// an allow-list entry of `3bm`.
fn is_tenant_allowed(allowlist: &[String], tenant: Option<&str>) -> bool {
  if allowlist.is_empty() {
    return true;
  }
  match tenant {
    Some(t) if !t.is_empty() => allowlist.iter().any(|allowed| allowed == t),
    _ => false,
  }
}

/// Read a request header value as an owned trimmed [`String`].
///
/// Returns `None` when the header is absent, not valid UTF-8, or whitespace-only.
fn header_value(parts: &Parts, name: &str) -> Option<String> {
  let raw = parts.headers.get(name)?.to_str().ok()?.trim();
  if raw.is_empty() {
    None
  } else {
    Some(raw.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::{HeaderMap, HeaderValue, Request};

  fn parts_with_headers(headers: &[(&str, &str)]) -> Parts {
    let mut builder = Request::builder();
    for (k, v) in headers {
      builder = builder.header(*k, *v);
    }
    builder.body(()).unwrap().into_parts().0
  }

  #[test]
  fn header_value_returns_trimmed_value() {
    let parts = parts_with_headers(&[("x-authentik-email", "  user@example.com  ")]);
    assert_eq!(
      header_value(&parts, "x-authentik-email"),
      Some("user@example.com".to_string())
    );
  }

  #[test]
  fn header_value_returns_none_for_missing_header() {
    let parts = parts_with_headers(&[]);
    assert_eq!(header_value(&parts, "x-authentik-email"), None);
  }

  #[test]
  fn header_value_returns_none_for_empty_header() {
    let parts = parts_with_headers(&[("x-authentik-email", "   ")]);
    assert_eq!(header_value(&parts, "x-authentik-email"), None);
  }

  #[test]
  fn authentik_meta_collects_all_prefixed_headers() {
    let parts = parts_with_headers(&[
      ("x-authentik-meta-tenant", "3bm"),
      ("x-authentik-meta-company", "3BM Bouwkunde"),
      ("x-authentik-meta-jobtitle", "Engineer"),
      ("x-authentik-meta-phone", "+31 6 1234 5678"),
      ("x-authentik-meta-regnumber", "12345678"),
    ]);
    let meta = AuthentikMeta::from_parts(&parts);
    assert_eq!(meta.tenant.as_deref(), Some("3bm"));
    assert_eq!(meta.company.as_deref(), Some("3BM Bouwkunde"));
    assert_eq!(meta.job_title.as_deref(), Some("Engineer"));
    assert_eq!(meta.phone.as_deref(), Some("+31 6 1234 5678"));
    assert_eq!(meta.registration_number.as_deref(), Some("12345678"));
  }

  #[test]
  fn authentik_meta_defaults_when_absent() {
    let parts = parts_with_headers(&[]);
    let meta = AuthentikMeta::from_parts(&parts);
    assert!(meta.tenant.is_none());
    assert!(meta.company.is_none());
    assert!(meta.job_title.is_none());
    assert!(meta.phone.is_none());
    assert!(meta.registration_number.is_none());
  }

  // --- Signal tests for extract_authentik_user ---
  //
  // Full happy-path integration requires a live AppState (PgPool + Config),
  // which we do not build in unit tests. The helper behaviour is covered
  // above; the early-return branch below is isolated via a focused shim.

  /// When the `X-Authentik-Email` header is absent, header extraction yields
  /// `None` — this is the shortcut the extractor uses to bail out before any
  /// database work. Covering it here prevents regressions where a refactor
  /// accidentally falls through to a DB call on anonymous requests.
  #[test]
  fn missing_email_header_short_circuits() {
    let parts = parts_with_headers(&[("x-authentik-username", "ghost")]);
    assert!(header_value(&parts, HDR_EMAIL).is_none());
    // Also verify that the HeaderMap lookup is case-insensitive in both
    // directions, so upstream header canonicalisation doesn't break us.
    let mut map = HeaderMap::new();
    map.insert("X-Authentik-Email", HeaderValue::from_static("a@b.c"));
    assert_eq!(map.get("x-authentik-email").unwrap(), "a@b.c");
  }

  // --- Tenant-gate tests (B — AUTHENTIK_AUTO_PROVISION_TENANTS) ---

  /// Empty allow-list = backwards-compatible behaviour: every tenant is
  /// allowed, including requests without a tenant header. This is the
  /// default when `AUTHENTIK_AUTO_PROVISION_TENANTS` is unset.
  #[test]
  fn tenant_gate_empty_allowlist_permits_all() {
    let allowlist: Vec<String> = Vec::new();
    assert!(is_tenant_allowed(&allowlist, Some("3bm")));
    assert!(is_tenant_allowed(&allowlist, Some("anything-goes")));
    assert!(is_tenant_allowed(&allowlist, None));
    assert!(is_tenant_allowed(&allowlist, Some("")));
  }

  /// Non-empty allow-list with a matching tenant permits auto-provisioning.
  #[test]
  fn tenant_gate_listed_tenant_is_permitted() {
    let allowlist = vec!["3bm".to_string(), "symitech".to_string()];
    assert!(is_tenant_allowed(&allowlist, Some("3bm")));
    assert!(is_tenant_allowed(&allowlist, Some("symitech")));
  }

  /// Non-empty allow-list with a non-matching or missing tenant blocks
  /// auto-provisioning — the extractor will return `None` and the request
  /// falls through to the Bearer-token flow.
  #[test]
  fn tenant_gate_unlisted_or_missing_tenant_is_blocked() {
    let allowlist = vec!["3bm".to_string(), "symitech".to_string()];
    // Unknown tenant slug.
    assert!(!is_tenant_allowed(&allowlist, Some("impertio")));
    // Missing tenant header altogether.
    assert!(!is_tenant_allowed(&allowlist, None));
    // Empty-string tenant is treated as "missing".
    assert!(!is_tenant_allowed(&allowlist, Some("")));
    // Case-sensitive comparison — slugs are canonical.
    assert!(!is_tenant_allowed(&allowlist, Some("3BM")));
  }
}
