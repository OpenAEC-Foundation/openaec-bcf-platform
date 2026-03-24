//! OIDC client — lightweight implementation using reqwest + jsonwebtoken.
//!
//! Discovers the OIDC provider, generates authorization URLs (with PKCE),
//! exchanges authorization codes for tokens, and validates ID tokens.

use base64::Engine;
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, DecodingKey, Validation};
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::models::user::OidcUserClaims;

/// OIDC provider discovery document.
#[derive(Debug, Clone, Deserialize)]
struct OidcDiscovery {
  issuer: String,
  authorization_endpoint: String,
  token_endpoint: String,
  jwks_uri: String,
}

/// Token response from the OIDC provider.
#[derive(Debug, Deserialize)]
struct TokenResponse {
  id_token: Option<String>,
}

/// Standard OIDC ID token claims.
#[derive(Debug, Deserialize)]
struct IdTokenClaims {
  sub: String,
  #[serde(default)]
  email: Option<String>,
  #[serde(default)]
  name: Option<String>,
  #[serde(default)]
  picture: Option<String>,
}

/// OIDC client that handles discovery, authorization, and token exchange.
#[derive(Clone)]
pub struct OidcClient {
  discovery: OidcDiscovery,
  jwks: JwkSet,
  client_id: String,
  client_secret: String,
  redirect_uri: String,
  http: reqwest::Client,
}

impl OidcClient {
  /// Discover the OIDC provider and fetch its JWKS.
  pub async fn discover(
    issuer_url: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
  ) -> Result<Self, OidcError> {
    let http = reqwest::Client::new();

    // Fetch discovery document
    let discovery_url = format!(
      "{}/.well-known/openid-configuration",
      issuer_url.trim_end_matches('/')
    );
    let discovery: OidcDiscovery = http
      .get(&discovery_url)
      .send()
      .await
      .map_err(|e| OidcError::Discovery(format!("failed to fetch discovery: {e}")))?
      .json()
      .await
      .map_err(|e| OidcError::Discovery(format!("invalid discovery document: {e}")))?;

    // Fetch JWKS
    let jwks: JwkSet = http
      .get(&discovery.jwks_uri)
      .send()
      .await
      .map_err(|e| OidcError::Discovery(format!("failed to fetch JWKS: {e}")))?
      .json()
      .await
      .map_err(|e| OidcError::Discovery(format!("invalid JWKS: {e}")))?;

    Ok(Self {
      discovery,
      jwks,
      client_id: client_id.to_string(),
      client_secret: client_secret.to_string(),
      redirect_uri: redirect_uri.to_string(),
      http,
    })
  }

  /// Generate the authorization URL with PKCE.
  ///
  /// Returns (auth_url, state, nonce, pkce_verifier).
  pub fn authorize_url(&self) -> (String, String, String, String) {
    let state = generate_random_string(32);
    let nonce = generate_random_string(32);
    let pkce_verifier = generate_random_string(64);

    // S256 PKCE challenge
    let challenge = {
      let hash = Sha256::digest(pkce_verifier.as_bytes());
      base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
    };

    let auth_url = format!(
      "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&nonce={}&code_challenge={}&code_challenge_method=S256",
      self.discovery.authorization_endpoint,
      urlencoding::encode(&self.client_id),
      urlencoding::encode(&self.redirect_uri),
      urlencoding::encode("openid email profile"),
      urlencoding::encode(&state),
      urlencoding::encode(&nonce),
      urlencoding::encode(&challenge),
    );

    (auth_url, state, nonce, pkce_verifier)
  }

  /// Exchange an authorization code for tokens and extract user claims.
  pub async fn exchange_code(
    &self,
    code: &str,
    pkce_verifier: &str,
    nonce: &str,
  ) -> Result<OidcUserClaims, OidcError> {
    // Exchange code for tokens
    let token_response: TokenResponse = self
      .http
      .post(&self.discovery.token_endpoint)
      .form(&[
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &self.redirect_uri),
        ("client_id", &self.client_id),
        ("client_secret", &self.client_secret),
        ("code_verifier", pkce_verifier),
      ])
      .send()
      .await
      .map_err(|e| OidcError::TokenExchange(format!("token request failed: {e}")))?
      .json()
      .await
      .map_err(|e| OidcError::TokenExchange(format!("invalid token response: {e}")))?;

    let id_token_str = token_response
      .id_token
      .ok_or_else(|| OidcError::TokenExchange("missing id_token in response".to_string()))?;

    // Validate the ID token
    let claims = self.validate_id_token(&id_token_str, nonce)?;

    Ok(OidcUserClaims {
      sub: claims.sub,
      email: claims.email.unwrap_or_default(),
      name: claims.name.unwrap_or_default(),
      avatar_url: claims.picture,
    })
  }

  /// Validate an ID token JWT against the provider's JWKS.
  fn validate_id_token(
    &self,
    token: &str,
    _nonce: &str,
  ) -> Result<IdTokenClaims, OidcError> {
    let header = decode_header(token)
      .map_err(|e| OidcError::TokenExchange(format!("invalid JWT header: {e}")))?;

    // Find the matching key in JWKS
    let kid = header
      .kid
      .as_ref()
      .ok_or_else(|| OidcError::TokenExchange("JWT has no kid".to_string()))?;

    let jwk = self
      .jwks
      .find(kid)
      .ok_or_else(|| OidcError::TokenExchange(format!("no matching JWK for kid: {kid}")))?;

    let decoding_key = DecodingKey::from_jwk(jwk)
      .map_err(|e| OidcError::TokenExchange(format!("invalid JWK: {e}")))?;

    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[&self.client_id]);
    validation.set_issuer(&[&self.discovery.issuer]);

    let token_data = decode::<IdTokenClaims>(token, &decoding_key, &validation)
      .map_err(|e| OidcError::TokenExchange(format!("ID token validation failed: {e}")))?;

    Ok(token_data.claims)
  }
}

/// Generate a cryptographically random URL-safe string.
fn generate_random_string(len: usize) -> String {
  let mut rng = rand::rng();
  let bytes: Vec<u8> = (0..len).map(|_| rng.random::<u8>()).collect();
  base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

/// OIDC-specific errors.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum OidcError {
  #[error("OIDC configuration error: {0}")]
  Config(String),
  #[error("OIDC discovery failed: {0}")]
  Discovery(String),
  #[error("token exchange failed: {0}")]
  TokenExchange(String),
}
