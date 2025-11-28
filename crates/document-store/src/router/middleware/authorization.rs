use std::{collections::HashMap, str::FromStr};

use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use once_cell::sync::Lazy;
use os_api::{
  AUTHORIZATION_HEADER, Claims, HttpError, INVALID_ERROR, REQUIRED_ERROR, authorization_from_header,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::router::entity::RouterState;

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
  let cache_key = format!("{}#{}", jku, kid);
  {
    let cache = JWK_CACHE.read().await;
    if let Some(key) = cache.get(&cache_key) {
      return Ok(key.clone());
    }
  }

  let resp = reqwest::get(jku)
    .await
    .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  if !resp.status().is_success() {
    return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
  }
  let json: serde_json::Value = resp
    .json()
    .await
    .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  let keys = json
    .get("keys")
    .and_then(|v| v.as_array())
    .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
  for jwk_value in keys {
    if jwk_value.get("kid").and_then(|v| v.as_str()) == Some(kid) {
      let jwk: Jwk = serde_json::from_value(jwk_value.clone())
        .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
      let decoding_key = DecodingKey::from_jwk(&jwk)
        .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;
      let mut cache = JWK_CACHE.write().await;
      cache.insert(cache_key, decoding_key.clone());
      return Ok(decoding_key);
    }
  }
  Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))
}

fn base64_url_decode(segment: &str) -> Result<Vec<u8>, HttpError> {
  use base64::Engine;
  use base64::engine::general_purpose::URL_SAFE_NO_PAD;
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

impl<S, T> FromRequestParts<S> for Authorization<T>
where
  RouterState: FromRef<S>,
  S: Send + Sync,
  T: Claims,
{
  type Rejection = HttpError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let _router_state = RouterState::from_ref(state);

    let authorization_header_value = parts
      .headers
      .get(AUTHORIZATION_HEADER)
      .ok_or_else(|| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR))?;
    let authorization_string = authorization_from_header(authorization_header_value)?;

    // Expect a JWT in the authorization string
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
    let validation = Validation::new(algorithm);
    let token_data = decode::<T>(token, &decoding_key, &validation)
      .map_err(|_| HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR))?;

    Ok(Self {
      claims: token_data.claims,
    })
  }
}
