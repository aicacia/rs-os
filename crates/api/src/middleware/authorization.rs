use std::{collections::HashMap, str::FromStr};

use axum::{extract::FromRequestParts, http::HeaderValue};
use http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, jwk::Jwk};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
  Claims,
  error::{HttpError, INVALID_ERROR, REQUIRED_ERROR},
};

pub const AUTHORIZATION_HEADER: &str = "Authorization";
pub const AUTHORIZATION_BEARER_PREFIX: &str = "Bearer ";

pub fn authorization_from_header(
  authorization_header_value: &HeaderValue,
) -> Result<&str, HttpError> {
  match authorization_header_value.to_str() {
    Ok(authorization_string) => {
      if authorization_string.len() < AUTHORIZATION_BEARER_PREFIX.len() {
        log::error!("invalid authorization header is too short");
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      if !authorization_string.starts_with(AUTHORIZATION_BEARER_PREFIX) {
        log::error!("authorization header does not start with 'Bearer '");
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      Ok(&authorization_string[AUTHORIZATION_BEARER_PREFIX.len()..])
    }
    Err(e) => {
      log::error!(
        "invalid authorization header cannot be parsed as string: {}",
        e
      );
      Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
    }
  }
}

pub struct Authorization<T>
where
  T: Claims,
{
  pub claims: T,
}

#[derive(Deserialize)]
struct RawJwtHeader {
  alg: String,
  #[serde(default)]
  kid: Option<String>,
  #[serde(default)]
  jku: Option<String>,
}

static JWK_CACHE: Lazy<RwLock<HashMap<String, DecodingKey>>> =
  Lazy::new(|| RwLock::new(HashMap::new()));

async fn fetch_decoding_key(jku: &str, kid: &str) -> Result<DecodingKey, HttpError> {
  let cache_key = format!("{}?kid={}", jku, kid);

  {
    let cache = JWK_CACHE.read().await;
    if let Some(key) = cache.get(&cache_key) {
      return Ok(key.clone());
    }
  }

  let resp = reqwest::get(jku).await.map_err(|e| {
    log::error!("failed to fetch JWK from {}: {}", jku, e);
    return HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR);
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
    return HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR);
  })?;
  let keys = json
    .get("keys")
    .and_then(|v| v.as_array())
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  for jwk_value in keys {
    if jwk_value.get("kid").and_then(|v| v.as_str()) == Some(kid) {
      let jwk: Jwk = serde_json::from_value(jwk_value.clone()).map_err(|e| {
        log::error!("failed to parse JWK from value: {}", e);
        HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
      })?;
      let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|e| {
        log::error!("failed to create decoding key from JWK: {}", e);
        HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
      })?;
      let mut cache = JWK_CACHE.write().await;
      cache.insert(cache_key, decoding_key.clone());
      return Ok(decoding_key);
    }
  }
  Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
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
  let token = authorization_string.trim();
  let mut segments = token.split('.');
  let header_segment = segments
    .next()
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  let _payload_segment = segments
    .next()
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  let _signature_segment = segments
    .next()
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;

  let raw_header = parse_header(header_segment)?;
  let kid = raw_header
    .kid
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  let jku = raw_header
    .jku
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  let algorithm = map_algorithm(&raw_header.alg)?;

  let decoding_key = fetch_decoding_key(&jku, &kid).await?;
  let mut validation = Validation::new(algorithm);
  validation.validate_aud = false;
  let token_data = decode::<T>(token, &decoding_key, &validation).map_err(|e| {
    log::warn!("failed to decode JWT: {}", e);
    HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)
  })?;

  Ok(token_data)
}

impl<S, T> FromRequestParts<S> for Authorization<T>
where
  S: Send + Sync,
  T: Claims,
{
  type Rejection = HttpError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let authorization_header_value = parts
      .headers
      .get(AUTHORIZATION_HEADER)
      .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR))?;
    let authorization_string = authorization_from_header(authorization_header_value)?;

    let token_data = parse_token_data::<T>(&authorization_string).await?;

    Ok(Self {
      claims: token_data.claims,
    })
  }
}
