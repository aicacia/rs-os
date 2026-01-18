use std::{
  str::FromStr,
  time::{Duration, Instant},
};

use axum::{extract::FromRequestParts, http::HeaderValue};
use dashmap::DashMap;
use hashbrown::HashMap;
use http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, jwk::Jwk};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
  Claims,
  error::{HttpError, INVALID_ERROR, REQUIRED_ERROR},
  util::permission_grants,
};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(bound(deserialize = "T: serde::de::DeserializeOwned"))]
pub struct UserInfo<T>
where
  T: Claims,
{
  #[serde(flatten)]
  pub claims: T,
  pub username: String,
  pub roles: HashMap<String, Vec<String>>,
  pub permissions: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct RawJwtHeader {
  alg: String,
  #[serde(default)]
  kid: Option<String>,
  #[serde(default)]
  jku: Option<String>,
}

pub struct Authorization<T>
where
  T: Claims,
{
  pub claims: T,
}

struct JwkEntry {
  decoding_key: DecodingKey,
  expires_at: Instant,
}

const CACHE_TTL: Duration = Duration::from_mins(5);
const CACHE_REFRESH_WINDOW: Duration = Duration::from_secs(30);

pub const AUTHORIZATION_HEADER: &str = "Authorization";
pub const AUTHORIZATION_BEARER_PREFIX: &str = "Bearer ";

pub fn authorization_from_header(
  authorization_header_value: &HeaderValue,
) -> Result<&str, HttpError> {
  log::debug!("parsing authorization header");
  match authorization_header_value.to_str() {
    Ok(authorization_string) => {
      if authorization_string.len() < AUTHORIZATION_BEARER_PREFIX.len() {
        log::warn!(
          "invalid authorization header is too short: length={}",
          authorization_string.len()
        );
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      if !authorization_string.starts_with(AUTHORIZATION_BEARER_PREFIX) {
        log::warn!(
          "authorization header does not start with 'Bearer ', starts with: {}",
          &authorization_string.chars().take(10).collect::<String>()
        );
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      log::debug!("authorization header parsed successfully");
      Ok(&authorization_string[AUTHORIZATION_BEARER_PREFIX.len()..])
    }
    Err(e) => {
      log::warn!(
        "invalid authorization header cannot be parsed as string: {}",
        e
      );
      Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
    }
  }
}

static JWK_CACHE: Lazy<DashMap<String, JwkEntry>> = Lazy::new(DashMap::new);

async fn fetch_jwk_from_server(jku: &str, kid: &str) -> Result<DecodingKey, HttpError> {
  log::debug!("fetching JWK from server: jku={}, kid={}", jku, kid);
  let resp = reqwest::get(jku).await.map_err(|e| {
    log::error!("failed to fetch JWK from {}: {}", jku, e);
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  if !resp.status().is_success() {
    log::error!(
      "failed to fetch JWK from {}: {}",
      jku,
      resp.text().await.unwrap_or_default()
    );
    return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
  }

  let json: serde_json::Value = resp.json().await.map_err(|e| {
    log::error!("failed to parse JWK JSON from {}: {}", jku, e);
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  let keys = json
    .get("keys")
    .and_then(|v| v.as_array())
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;

  for jwk_value in keys {
    if jwk_value.get("kid").and_then(|v| v.as_str()) == Some(kid) {
      log::debug!("found matching JWK for kid={}", kid);
      let jwk: Jwk = serde_json::from_value(jwk_value.clone()).map_err(|e| {
        log::error!("failed to parse JWK from value: {}", e);
        HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
      })?;
      let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|e| {
        log::error!("failed to create decoding key from JWK: {}", e);
        HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
      })?;
      log::debug!("successfully created decoding key for kid={}", kid);
      return Ok(decoding_key);
    }
  }

  log::warn!(
    "JWK not found for kid={} in {} keys from {}",
    kid,
    keys.len(),
    jku
  );
  Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
}

async fn fetch_decoding_key(jku: &str, kid: &str) -> Result<DecodingKey, HttpError> {
  let cache_key = format!("{}?kid={}", jku, kid);

  let now = Instant::now();

  if let Some(entry) = JWK_CACHE.get(&cache_key) {
    let remaining = entry.expires_at.saturating_duration_since(now);

    if remaining.is_zero() {
      log::debug!("JWK cache entry expired for {}", cache_key);
      drop(entry);
    } else {
      log::debug!(
        "JWK cache hit for {}, remaining: {:?}",
        cache_key,
        remaining
      );
      if remaining <= CACHE_REFRESH_WINDOW {
        log::debug!("JWK cache entry within refresh window, spawning background refresh");
        let cache_key = cache_key.clone();
        let jku = jku.to_owned();
        let kid = kid.to_owned();

        tokio::spawn(async move {
          if let Ok(decoding_key) = fetch_jwk_from_server(&jku, &kid).await {
            let expires_at = Instant::now() + CACHE_TTL;
            JWK_CACHE.insert(
              cache_key,
              JwkEntry {
                decoding_key,
                expires_at,
              },
            );
          } else {
            log::warn!("failed to refresh JWK for {}?kid={}", jku, kid);
          }
        });
      }

      return Ok(entry.decoding_key.clone());
    }
  }

  log::debug!("JWK cache miss for {}, fetching from server", cache_key);
  let decoding_key = fetch_jwk_from_server(jku, kid).await?;
  let expires_at = Instant::now() + CACHE_TTL;
  JWK_CACHE.insert(
    cache_key,
    JwkEntry {
      decoding_key: decoding_key.clone(),
      expires_at,
    },
  );

  Ok(decoding_key)
}

fn base64_url_decode(segment: &str) -> Result<Vec<u8>, HttpError> {
  use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
  URL_SAFE_NO_PAD
    .decode(segment)
    .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
}

fn parse_header(segment: &str) -> Result<RawJwtHeader, HttpError> {
  let bytes = base64_url_decode(segment)?;
  serde_json::from_slice::<RawJwtHeader>(&bytes)
    .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
}

fn map_algorithm(alg: &str) -> Result<Algorithm, HttpError> {
  Algorithm::from_str(alg)
    .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
}

pub async fn parse_token_data<T: Claims>(
  authorization_string: &str,
) -> Result<jsonwebtoken::TokenData<T>, HttpError> {
  log::debug!("parsing token data");
  let token = authorization_string.trim();
  let mut segments = token.split('.');
  let header_segment = segments.next().ok_or_else(|| {
    log::warn!("token missing header segment");
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;
  let _payload_segment = segments.next().ok_or_else(|| {
    log::warn!("token missing payload segment");
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;
  let _signature_segment = segments.next().ok_or_else(|| {
    log::warn!("token missing signature segment");
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  let raw_header = parse_header(header_segment)?;
  log::debug!(
    "parsed token header: alg={}, kid={:?}, jku={:?}",
    raw_header.alg,
    raw_header.kid,
    raw_header.jku
  );
  let kid = raw_header.kid.ok_or_else(|| {
    log::warn!("token header missing kid");
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;
  let jku = raw_header.jku.ok_or_else(|| {
    log::warn!("token header missing jku");
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;
  let algorithm = map_algorithm(&raw_header.alg)?;

  let decoding_key = fetch_decoding_key(&jku, &kid).await?;
  let mut validation = Validation::new(algorithm);
  validation.validate_aud = false;
  log::debug!("attempting to decode JWT with algorithm={:?}", algorithm);
  let token_data = decode::<T>(token, &decoding_key, &validation).map_err(|e| {
    log::warn!("failed to decode JWT: {}", e);
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  log::debug!("successfully decoded JWT");
  Ok(token_data)
}

impl<S, T> FromRequestParts<S> for Authorization<T>
where
  S: Send + Sync,
  T: Claims,
{
  type Rejection = HttpError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    log::debug!("extracting Authorization from request");
    let authorization_header_value = parts.headers.get(AUTHORIZATION_HEADER).ok_or_else(|| {
      log::warn!("authorization header missing from request");
      HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR)
    })?;
    let authorization_string = authorization_from_header(authorization_header_value)?;

    let token_data = parse_token_data::<T>(authorization_string).await?;

    log::debug!("successfully extracted Authorization");
    Ok(Self {
      claims: token_data.claims,
    })
  }
}

pub struct UserAuthorization<T>
where
  T: Claims,
{
  pub user_info: UserInfo<T>,
}

impl<T: Claims> UserAuthorization<T> {
  fn has_permission_for(
    permissions: &HashMap<String, Vec<String>>,
    application_urn: &str,
    required_permission: &str,
  ) -> bool {
    if let Some(user_permissions) = permissions.get(application_urn) {
      return user_permissions
        .iter()
        .any(|user_permission| permission_grants(user_permission, required_permission));
    }

    if let Some(user_permissions) = permissions.get("*") {
      return user_permissions
        .iter()
        .any(|user_permission| permission_grants(user_permission, required_permission));
    }

    false
  }

  pub fn has_permission(&self, application_urn: &str, permission: &str) -> Result<(), HttpError> {
    if Self::has_permission_for(&self.user_info.permissions, application_urn, permission) {
      return Ok(());
    }
    Err(HttpError::forbidden().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
  }

  pub fn has_permissions(
    &self,
    application_urn: &str,
    permissions: &[&str],
  ) -> Result<(), HttpError> {
    if permissions.iter().all(|permission| {
      Self::has_permission_for(&self.user_info.permissions, application_urn, permission)
    }) {
      return Ok(());
    }
    Err(HttpError::forbidden().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
  }
}

#[cfg(test)]
mod tests {
  use hashbrown::HashMap;

  use super::*;
  use crate::claims::BasicClaims;

  #[test]
  fn has_permission_for_scoped_and_global() {
    let mut permissions: HashMap<String, Vec<String>> = HashMap::new();
    permissions.insert("fs".to_string(), vec!["fs:*".to_string()]);
    permissions.insert("*".to_string(), vec!["os:*".to_string()]);

    assert!(UserAuthorization::<BasicClaims>::has_permission_for(
      &permissions,
      "fs",
      "fs:read"
    ));
    assert!(UserAuthorization::<BasicClaims>::has_permission_for(
      &permissions,
      "os",
      "os:oidc"
    ));
    assert!(!UserAuthorization::<BasicClaims>::has_permission_for(
      &permissions,
      "fs",
      "os:read"
    ));
  }
}

#[derive(Deserialize)]
struct OidcConfiguration {
  userinfo_endpoint: String,
}

struct OidcConfigEntry {
  userinfo_endpoint: String,
  expires_at: Instant,
}

static OIDC_CONFIG_CACHE: Lazy<DashMap<String, OidcConfigEntry>> = Lazy::new(DashMap::new);

async fn fetch_oidc_configuration(issuer: &str) -> Result<String, HttpError> {
  log::debug!("fetching OIDC configuration for issuer: {}", issuer);
  let cache_key = issuer.to_string();
  let now = Instant::now();

  if let Some(entry) = OIDC_CONFIG_CACHE.get(&cache_key) {
    let remaining = entry.expires_at.saturating_duration_since(now);
    if !remaining.is_zero() {
      log::debug!("OIDC config cache hit for issuer: {}", issuer);
      return Ok(entry.userinfo_endpoint.clone());
    }
    log::debug!("OIDC config cache entry expired for issuer: {}", issuer);
    drop(entry);
  }

  let issuer_trimmed = issuer.trim_end_matches('/');
  let well_known_url = format!("{}/.well-known/openid-configuration", issuer_trimmed);

  log::debug!("fetching OIDC well-known config from: {}", well_known_url);
  let resp = reqwest::get(&well_known_url).await.map_err(|e| {
    log::error!(
      "failed to fetch OIDC configuration from {}: {}",
      well_known_url,
      e
    );
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  if !resp.status().is_success() {
    log::error!(
      "failed to fetch OIDC configuration from {}: {}",
      well_known_url,
      resp.text().await.unwrap_or_default()
    );
    return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
  }

  let config: OidcConfiguration = resp.json().await.map_err(|e| {
    log::error!(
      "failed to parse OIDC configuration JSON from {}: {}",
      well_known_url,
      e
    );
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  let userinfo_endpoint = config.userinfo_endpoint.clone();
  let expires_at = Instant::now() + CACHE_TTL;
  OIDC_CONFIG_CACHE.insert(
    cache_key,
    OidcConfigEntry {
      userinfo_endpoint: userinfo_endpoint.clone(),
      expires_at,
    },
  );

  Ok(userinfo_endpoint)
}

async fn fetch_user_info<T: Claims>(
  authorization_string: &str,
  issuer: &str,
) -> Result<UserInfo<T>, HttpError> {
  log::debug!("fetching user info for issuer: {}", issuer);
  let userinfo_endpoint = fetch_oidc_configuration(issuer).await?;

  log::debug!("userinfo endpoint: {}", userinfo_endpoint);
  let client = reqwest::Client::new();
  let resp = client
    .get(&userinfo_endpoint)
    .header(
      AUTHORIZATION_HEADER,
      format!("{}{}", AUTHORIZATION_BEARER_PREFIX, authorization_string),
    )
    .send()
    .await
    .map_err(|e| {
      log::error!(
        "failed to fetch user info from {}: {}",
        userinfo_endpoint,
        e
      );
      HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
    })?;

  if !resp.status().is_success() {
    log::error!(
      "failed to fetch user info from {}: {}",
      userinfo_endpoint,
      resp.text().await.unwrap_or_default()
    );
    return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
  }

  let user_info: UserInfo<T> = resp.json().await.map_err(|e| {
    log::error!("failed to parse user info JSON: {}", e);
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  log::debug!(
    "successfully fetched user info for username: {}",
    user_info.username
  );
  Ok(user_info)
}

impl<S, T> FromRequestParts<S> for UserAuthorization<T>
where
  S: Send + Sync,
  T: Claims,
{
  type Rejection = HttpError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    log::debug!("extracting UserAuthorization from request");
    let authorization_header_value = parts.headers.get(AUTHORIZATION_HEADER).ok_or_else(|| {
      log::warn!("authorization header missing from request for UserAuthorization");
      HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR)
    })?;
    let authorization_string = authorization_from_header(authorization_header_value)?;

    let token_data = parse_token_data::<T>(authorization_string).await?;
    let issuer = token_data.claims.iss();
    log::debug!("token parsed, issuer: {}", issuer);

    let user_info = fetch_user_info::<T>(authorization_string, issuer).await?;

    log::info!(
      "fetched user info for issuer {}: username={}, roles: {:?}, permissions: {:?}",
      issuer,
      user_info.username,
      user_info.roles,
      user_info.permissions
    );

    Ok(Self { user_info })
  }
}
