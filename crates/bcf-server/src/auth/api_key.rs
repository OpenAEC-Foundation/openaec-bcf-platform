//! API key generation and validation.
//!
//! Keys have the format `bcfk_<32 random chars>`.
//! The first 8 characters after the prefix serve as a lookup index.

use rand::Rng;

/// API key prefix.
const KEY_PREFIX: &str = "bcfk_";

/// Length of the random part of the key.
const KEY_RANDOM_LEN: usize = 32;

/// Number of prefix chars stored for lookup (after `bcfk_`).
const LOOKUP_PREFIX_LEN: usize = 8;

/// bcrypt cost factor.
const BCRYPT_COST: u32 = 10;

/// Generated API key with its hash and lookup prefix.
pub struct GeneratedApiKey {
  /// The full raw key (shown to user once).
  pub raw_key: String,
  /// bcrypt hash of the raw key.
  pub key_hash: String,
  /// First 8 chars after `bcfk_` for database lookup.
  pub prefix: String,
}

/// Generate a new API key.
pub fn generate_api_key() -> Result<GeneratedApiKey, bcrypt::BcryptError> {
  let mut rng = rand::rng();
  let random_part: String = (0..KEY_RANDOM_LEN)
    .map(|_| {
      let idx = rng.random_range(0..36u32);
      if idx < 10 {
        (b'0' + idx as u8) as char
      } else {
        (b'a' + (idx - 10) as u8) as char
      }
    })
    .collect();

  let raw_key = format!("{KEY_PREFIX}{random_part}");
  let prefix = format!("{KEY_PREFIX}{}", &random_part[..LOOKUP_PREFIX_LEN]);
  let key_hash = bcrypt::hash(&raw_key, BCRYPT_COST)?;

  Ok(GeneratedApiKey {
    raw_key,
    key_hash,
    prefix,
  })
}

/// Verify a raw API key against a bcrypt hash.
pub fn verify_api_key(raw_key: &str, hash: &str) -> bool {
  bcrypt::verify(raw_key, hash).unwrap_or(false)
}

/// Extract the lookup prefix from a raw API key.
pub fn extract_prefix(raw_key: &str) -> Option<String> {
  if let Some(rest) = raw_key.strip_prefix(KEY_PREFIX) {
    if rest.len() >= LOOKUP_PREFIX_LEN {
      return Some(format!("{KEY_PREFIX}{}", &rest[..LOOKUP_PREFIX_LEN]));
    }
  }
  None
}
