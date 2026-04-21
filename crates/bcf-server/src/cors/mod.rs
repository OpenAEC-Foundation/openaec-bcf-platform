//! Tenant-aware CORS layer builder.
//!
//! Reads `<tenants_root>/<slug>/tenant.yaml` bij startup, bouwt een
//! origin-set, en construeert een tower-http [`CorsLayer`] die per
//! request de `Origin`-header matcht tegen deze set.
//!
//! Patroon analoog aan `openaec-reports` `core/tenant_cors.py` +
//! `core/cors_middleware.py` (commit `66fb4f7`). Schema-referentie:
//! `C:/GitHub/openaec-tenants/tenants/_schema.md`.
//!
//! # Semantiek
//!
//! - Elke `<tenants_root>/<slug>/` sub-directory wordt gescand.
//! - `tenant.yaml` ontbreekt of niet-parsable → `WARN` + skip tenant
//!   (behandeld als `active: false`).
//! - `active: false` → tenant geheel overgeslagen.
//! - Per-origin validatie: moet `http(s)://` prefix, geen trailing slash,
//!   geen wildcards. Invalide entries worden per-origin geskipped met
//!   `WARN`; de rest van de tenant wordt gewoon geladen.
//! - `include_dev=true` (niet-productie): union met `allowed_origins_dev`.
//!
//! Bij lege uiteindelijke origin-set wordt teruggevallen op
//! `CORS_ORIGINS`-env (komma-gescheiden); is die ook leeg dan
//! `AllowOrigin::Any` met een startup-warning.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::http::{HeaderName, Method};
use serde::Deserialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Maximum preflight cache (seconden).
const PREFLIGHT_MAX_AGE_SECS: u64 = 600;

/// Toegestane protocol-prefixes voor een origin-string.
const VALID_PROTOCOLS: &[&str] = &["http://", "https://"];

/// Raw vorm van `tenant.yaml` — alleen de velden die we hier nodig hebben.
#[derive(Debug, Deserialize)]
struct TenantYaml {
  /// Canonical slug (matcht dir-naam). Niet strikt nodig voor CORS maar
  /// handig voor logging bij mismatch.
  #[allow(dead_code)]
  slug: Option<String>,
  /// `false` → tenant skippen. Ontbrekend veld = `true` (default).
  active: Option<bool>,
  /// CORS-block. Ontbreken = tenant laadt zonder origins (niks erg).
  cors: Option<CorsSection>,
}

/// CORS-subblok uit `tenant.yaml`.
#[derive(Debug, Deserialize)]
struct CorsSection {
  /// Productie-origins (altijd geladen).
  allowed_origins: Option<Vec<String>>,
  /// Dev/local origins (alleen bij `include_dev=true`).
  allowed_origins_dev: Option<Vec<String>>,
}

/// Valideer één origin-string volgens schema-regels.
///
/// Retourneert `false` bij overtreding, logt een `WARN` met de slug-context.
fn validate_origin(origin: &str, slug: &str) -> bool {
  if origin.is_empty() {
    tracing::warn!(tenant = slug, "lege origin genegeerd");
    return false;
  }
  if !VALID_PROTOCOLS.iter().any(|p| origin.starts_with(p)) {
    tracing::warn!(
      tenant = slug,
      origin = origin,
      "origin heeft geen http(s):// protocol — skip"
    );
    return false;
  }
  if origin.ends_with('/') {
    tracing::warn!(
      tenant = slug,
      origin = origin,
      "origin eindigt op trailing slash — skip"
    );
    return false;
  }
  if origin.contains('*') {
    tracing::warn!(
      tenant = slug,
      origin = origin,
      "origin bevat wildcard — skip (niet ondersteund)"
    );
    return false;
  }
  if origin != origin.to_ascii_lowercase() {
    tracing::warn!(
      tenant = slug,
      origin = origin,
      "origin is niet lowercase — skip"
    );
    return false;
  }
  true
}

/// Laad één `tenant.yaml` en voeg gevalideerde origins toe aan `out`.
///
/// Retourneert `true` als de tenant actief was en meegenomen is. `false` bij
/// skip (malformed, inactive, etc.).
fn load_one(tenant_dir: &Path, include_dev: bool, out: &mut HashSet<String>) -> bool {
  let dir_slug = tenant_dir
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("<unknown>")
    .to_string();

  let yaml_path = tenant_dir.join("tenant.yaml");
  if !yaml_path.exists() {
    tracing::warn!(
      tenant = %dir_slug,
      "geen tenant.yaml gevonden — skip (CORS-config niet geladen)"
    );
    return false;
  }

  let content = match std::fs::read_to_string(&yaml_path) {
    Ok(c) => c,
    Err(err) => {
      tracing::warn!(
        tenant = %dir_slug,
        error = %err,
        "tenant.yaml niet leesbaar — behandel als inactive"
      );
      return false;
    }
  };

  let parsed: TenantYaml = match serde_yml::from_str(&content) {
    Ok(p) => p,
    Err(err) => {
      tracing::warn!(
        tenant = %dir_slug,
        error = %err,
        "tenant.yaml is malformed — behandel als inactive"
      );
      return false;
    }
  };

  if !parsed.active.unwrap_or(true) {
    tracing::info!(tenant = %dir_slug, "active=false — skip");
    return false;
  }

  let slug_for_log = parsed.slug.as_deref().unwrap_or(&dir_slug);

  let Some(cors_block) = parsed.cors else {
    tracing::info!(
      tenant = %slug_for_log,
      "geen cors-block — tenant actief maar zonder CORS-origins"
    );
    return true;
  };

  let mut added = 0_usize;

  if let Some(list) = cors_block.allowed_origins {
    for origin in list {
      if validate_origin(&origin, slug_for_log) && out.insert(origin) {
        added += 1;
      }
    }
  }

  if include_dev {
    if let Some(list) = cors_block.allowed_origins_dev {
      for origin in list {
        if validate_origin(&origin, slug_for_log) && out.insert(origin) {
          added += 1;
        }
      }
    }
  }

  tracing::info!(
    tenant = %slug_for_log,
    added = added,
    "tenant geladen"
  );
  true
}

/// Scan `tenants_root` voor sub-directories en bouw de union van alle
/// toegestane origins.
///
/// Directories met een leidende `.` of `_` (verborgen of schema/docs) worden
/// overgeslagen. Zie module-doc voor volledige fallback-semantiek.
pub fn load_tenant_origins(tenants_root: &Path, include_dev: bool) -> HashSet<String> {
  let mut origins: HashSet<String> = HashSet::new();

  let read_dir = match std::fs::read_dir(tenants_root) {
    Ok(rd) => rd,
    Err(err) => {
      tracing::warn!(
        tenants_root = %tenants_root.display(),
        error = %err,
        "tenants_root niet leesbaar — geen tenants geladen"
      );
      return origins;
    }
  };

  let mut entries: Vec<_> = read_dir
    .filter_map(Result::ok)
    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
    .collect();
  entries.sort_by_key(|e| e.file_name());

  for entry in entries {
    let name = entry.file_name();
    let name_str = name.to_string_lossy();
    if name_str.starts_with('.') || name_str.starts_with('_') {
      continue;
    }
    load_one(&entry.path(), include_dev, &mut origins);
  }

  origins
}

/// Parse een komma-gescheiden env-fallback naar een origin-set. Lege strings
/// worden genegeerd en validatie loopt langs [`validate_origin`].
fn parse_env_fallback(raw: &str) -> HashSet<String> {
  raw
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .filter(|s| validate_origin(s, "env:CORS_ORIGINS"))
    .map(str::to_string)
    .collect()
}

/// Request headers die de browser mag meesturen in een cross-origin call.
///
/// `AUTHORIZATION` + `CONTENT_TYPE` dekken standaard XHR. De
/// `x-authentik-*` / `x-original-tenant` entries komen van de forward_auth
/// chain (zie `auth/forward_auth.rs`) — die stuurt Caddy, maar bij dev/CLI
/// testen kan de frontend deze ook expliciet zetten.
fn allowed_request_headers() -> Vec<HeaderName> {
  vec![
    axum::http::header::AUTHORIZATION,
    axum::http::header::CONTENT_TYPE,
    axum::http::header::ACCEPT,
    HeaderName::from_static("x-authentik-email"),
    HeaderName::from_static("x-authentik-username"),
    HeaderName::from_static("x-authentik-name"),
    HeaderName::from_static("x-authentik-uid"),
    HeaderName::from_static("x-authentik-groups"),
    HeaderName::from_static("x-authentik-meta-tenant"),
    HeaderName::from_static("x-authentik-meta-company"),
    HeaderName::from_static("x-authentik-meta-jobtitle"),
    HeaderName::from_static("x-authentik-meta-phone"),
    HeaderName::from_static("x-authentik-meta-regnumber"),
    HeaderName::from_static("x-original-tenant"),
  ]
}

/// Bouw een [`CorsLayer`] op basis van een origin-set + optionele env-fallback.
///
/// # Fallback-gedrag
///
/// 1. `origins` niet-leeg → `AllowOrigin::predicate` met exact-match.
/// 2. `origins` leeg + `fallback_env` parseable → gebruik de env-lijst.
/// 3. Beide leeg → `AllowOrigin::Any` met een startup-`warn` (effectief
///    hetzelfde gedrag als voor B-4 uitrol — Caddy blijft gate-keeper).
///
/// Methods zijn een expliciete lijst (geen `Any`), credentials staan aan.
/// Dat is vereist: browsers weigeren credentials met wildcard-origin, dus de
/// predicate-path echo't de concrete Origin-header terug.
pub fn build_cors_layer(origins: HashSet<String>, fallback_env: Option<String>) -> CorsLayer {
  let effective = if origins.is_empty() {
    match fallback_env {
      Some(raw) if !raw.trim().is_empty() => {
        let parsed = parse_env_fallback(&raw);
        if parsed.is_empty() {
          tracing::warn!(
            "CORS: geen tenant-origins en CORS_ORIGINS leverde 0 geldige entries — permissive fallback actief"
          );
          HashSet::new()
        } else {
          tracing::info!(
            count = parsed.len(),
            "CORS: {} origin(s) geladen uit CORS_ORIGINS env-fallback",
            parsed.len()
          );
          parsed
        }
      }
      _ => {
        tracing::warn!(
          "CORS: geen tenant-origins en geen CORS_ORIGINS env — permissive fallback (AllowOrigin::Any) actief"
        );
        HashSet::new()
      }
    }
  } else {
    tracing::info!(
      count = origins.len(),
      "CORS: {} tenant-origin(s) geladen uit tenant.yaml",
      origins.len()
    );
    origins
  };

  let methods = [
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::DELETE,
    Method::OPTIONS,
    Method::PATCH,
  ];

  let max_age = Duration::from_secs(PREFLIGHT_MAX_AGE_SECS);

  if effective.is_empty() {
    // Permissive path — geen credentials mogelijk icm Any, maar dat is
    // consistent met gedrag voor B-4 (Caddy deed de gate).
    return CorsLayer::new()
      .allow_origin(AllowOrigin::any())
      .allow_methods(methods)
      .allow_headers(allowed_request_headers())
      .max_age(max_age);
  }

  let origin_set: Arc<HashSet<String>> = Arc::new(effective);
  let predicate_set = Arc::clone(&origin_set);
  let allow_origin = AllowOrigin::predicate(move |origin, _req| {
    origin
      .to_str()
      .map(|o| predicate_set.contains(o))
      .unwrap_or(false)
  });

  CorsLayer::new()
    .allow_origin(allow_origin)
    .allow_methods(methods)
    .allow_headers(allowed_request_headers())
    .allow_credentials(true)
    .max_age(max_age)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  /// Schrijf `tenant.yaml` in `root/<slug>/`.
  fn write_tenant(root: &Path, slug: &str, body: &str) {
    let dir = root.join(slug);
    fs::create_dir_all(&dir).expect("mkdir");
    fs::write(dir.join("tenant.yaml"), body).expect("write yaml");
  }

  #[test]
  fn test_load_tenant_origins_skips_inactive() {
    let tmp = TempDir::new().unwrap();
    write_tenant(
      tmp.path(),
      "alpha",
      "slug: alpha\nactive: true\ncors:\n  allowed_origins:\n    - https://alpha.example.com\n  allowed_origins_dev: []\n",
    );
    write_tenant(
      tmp.path(),
      "beta",
      "slug: beta\nactive: false\ncors:\n  allowed_origins:\n    - https://beta.example.com\n  allowed_origins_dev: []\n",
    );

    let origins = load_tenant_origins(tmp.path(), false);
    assert!(origins.contains("https://alpha.example.com"));
    assert!(!origins.contains("https://beta.example.com"));
  }

  #[test]
  fn test_load_tenant_origins_skips_malformed_yaml() {
    let tmp = TempDir::new().unwrap();
    write_tenant(
      tmp.path(),
      "broken",
      "slug: broken\nactive: true\ncors: [this is: not valid",
    );
    write_tenant(
      tmp.path(),
      "ok",
      "slug: ok\nactive: true\ncors:\n  allowed_origins:\n    - https://ok.example.com\n  allowed_origins_dev: []\n",
    );

    let origins = load_tenant_origins(tmp.path(), false);
    // Broken tenant mag niks toevoegen, ok tenant wel.
    assert_eq!(origins.len(), 1);
    assert!(origins.contains("https://ok.example.com"));
  }

  #[test]
  fn test_origin_validation_skips_invalid_per_entry() {
    let tmp = TempDir::new().unwrap();
    write_tenant(
      tmp.path(),
      "mixed",
      "slug: mixed\nactive: true\ncors:\n  allowed_origins:\n    - https://good.example.com\n    - https://trailing.example.com/\n    - https://*.wildcard.example.com\n    - example.com\n    - https://GOOD.example.com\n  allowed_origins_dev: []\n",
    );

    let origins = load_tenant_origins(tmp.path(), false);
    // Alleen de éérste (lowercase, geen trailing, geen wildcard, met protocol) blijft.
    assert_eq!(origins.len(), 1);
    assert!(origins.contains("https://good.example.com"));
    assert!(!origins.contains("https://trailing.example.com/"));
    assert!(!origins.contains("https://*.wildcard.example.com"));
    assert!(!origins.contains("example.com"));
    assert!(!origins.contains("https://GOOD.example.com"));
  }

  #[test]
  fn test_include_dev_flag() {
    let tmp = TempDir::new().unwrap();
    write_tenant(
      tmp.path(),
      "t",
      "slug: t\nactive: true\ncors:\n  allowed_origins:\n    - https://prod.example.com\n  allowed_origins_dev:\n    - http://localhost:5173\n",
    );

    let prod_only = load_tenant_origins(tmp.path(), false);
    assert_eq!(prod_only.len(), 1);
    assert!(prod_only.contains("https://prod.example.com"));
    assert!(!prod_only.contains("http://localhost:5173"));

    let with_dev = load_tenant_origins(tmp.path(), true);
    assert_eq!(with_dev.len(), 2);
    assert!(with_dev.contains("https://prod.example.com"));
    assert!(with_dev.contains("http://localhost:5173"));
  }

  #[test]
  fn test_load_skips_underscore_and_dot_dirs() {
    let tmp = TempDir::new().unwrap();
    // Underscore dir simuleert `_schema.md`-achtige meta-dir.
    write_tenant(
      tmp.path(),
      "_meta",
      "slug: _meta\nactive: true\ncors:\n  allowed_origins:\n    - https://meta.example.com\n  allowed_origins_dev: []\n",
    );
    write_tenant(
      tmp.path(),
      "real",
      "slug: real\nactive: true\ncors:\n  allowed_origins:\n    - https://real.example.com\n  allowed_origins_dev: []\n",
    );

    let origins = load_tenant_origins(tmp.path(), false);
    assert!(origins.contains("https://real.example.com"));
    assert!(!origins.contains("https://meta.example.com"));
  }

  #[test]
  fn test_load_missing_tenant_yaml_skipped() {
    let tmp = TempDir::new().unwrap();
    // Maak dir maar geen tenant.yaml.
    fs::create_dir_all(tmp.path().join("empty")).unwrap();
    write_tenant(
      tmp.path(),
      "filled",
      "slug: filled\nactive: true\ncors:\n  allowed_origins:\n    - https://filled.example.com\n  allowed_origins_dev: []\n",
    );

    let origins = load_tenant_origins(tmp.path(), false);
    assert_eq!(origins.len(), 1);
    assert!(origins.contains("https://filled.example.com"));
  }

  #[test]
  fn test_build_cors_layer_with_empty_falls_back_to_env() {
    // Env bevat 1 geldige + 1 invalide origin.
    let layer = build_cors_layer(
      HashSet::new(),
      Some("https://env.example.com, not-a-url".to_string()),
    );
    // We kunnen de interne predicate niet direct inspecteren, maar de layer
    // moet bouwen zonder panic. Integratie-test gebeurt in main.
    let _ = layer;
  }

  #[test]
  fn test_build_cors_layer_with_origins() {
    let mut set = HashSet::new();
    set.insert("https://a.example.com".to_string());
    set.insert("https://b.example.com".to_string());
    let layer = build_cors_layer(set, None);
    // Smoketest: layer bouwt zonder panic met 2 origins.
    let _ = layer;
  }

  #[test]
  fn test_build_cors_layer_empty_and_no_env_is_permissive() {
    let layer = build_cors_layer(HashSet::new(), None);
    // Smoketest: permissive-fallback pad moet bouwen.
    let _ = layer;
  }

  /// Integratietest tegen de echte `openaec-tenants` checkout. Alleen
  /// nuttig op developer-machines waar die repo lokaal staat; daarom
  /// `#[ignore]` zodat CI hem overslaat.
  ///
  /// Draai met: `cargo test -p bcf-server -- --ignored load_real_tenants`
  #[test]
  #[ignore]
  fn load_real_tenants_checkout() {
    let root = Path::new("C:/GitHub/openaec-tenants/tenants");
    if !root.exists() {
      eprintln!("skip: {} bestaat niet op deze machine", root.display());
      return;
    }
    let origins = load_tenant_origins(root, true);
    // Verwacht: 3bm (3 prod + 3 dev) + symitech (2 prod + 2 dev) +
    // openaec_foundation (0 prod + 2 dev). test_tenant_brand is
    // active: false → geskipt.
    assert!(origins.contains("https://report.open-aec.com"));
    assert!(origins.contains("https://cloud-3bm.open-aec.com"));
    assert!(origins.contains("https://mockup.symitech.nl"));
    assert!(origins.contains("https://va.symitech.nl"));
    assert!(origins.contains("http://localhost:5173"));
  }
}
