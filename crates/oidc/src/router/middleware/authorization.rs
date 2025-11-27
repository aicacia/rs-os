use axum::extract::{FromRef, FromRequestParts};
use http::{HeaderValue, request::Parts};
use jsonwebtoken::DecodingKey;

use crate::{
  core::{
    config::app_config::AppConfig,
    jwk::sql::{JwkSQLRow, get_jwk_by_kid},
  },
  model::revoked_token::sql::is_token_revoked,
  router::{
    common::{
      constants::{AUTHORIZATION_HEADER, TOKEN_TYPE_BEARER},
      entity::Claims,
      helper::parse_jwt,
    },
    entity::RouterState,
    error::{HttpError, INVALID_ERROR, REQUIRED_ERROR},
  },
};

pub struct Authorization<T>
where
  T: Claims,
{
  pub claims: T,
}

impl<S, T> FromRequestParts<S> for Authorization<T>
where
  RouterState: FromRef<S>,
  S: Send + Sync,
  T: Claims,
{
  type Rejection = HttpError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let router_state = RouterState::from_ref(state);

    if let Some(authorization_header_value) = parts.headers.get(AUTHORIZATION_HEADER) {
      let authorization_string = authorization_from_header(authorization_header_value)?;
      let (token_data, _jwk_sql_row) = parse_authorization(
        &router_state.pool,
        &router_state.config,
        authorization_string,
      )
      .await?;

      return Ok(Self {
        claims: token_data.claims,
      });
    }
    Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR))
  }
}

pub fn authorization_from_header(
  authorization_header_value: &HeaderValue,
) -> Result<&str, HttpError> {
  match authorization_header_value.to_str() {
    Ok(authorization_string) => {
      if authorization_string.len() < TOKEN_TYPE_BEARER.len() + 1 {
        log::error!("invalid authorization header is invalid");
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      Ok(&authorization_string[(TOKEN_TYPE_BEARER.len() + 1)..])
    }
    Err(e) => {
      log::error!("invalid authorization header is invalid: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  }
}

pub async fn parse_authorization<T>(
  pool: &sqlx::AnyPool,
  app_config: &AppConfig,
  authorization_string: &str,
) -> Result<(jsonwebtoken::TokenData<T>, JwkSQLRow), HttpError>
where
  T: Claims,
{
  let header = match jsonwebtoken::decode_header(authorization_string) {
    Ok(header) => header,
    Err(e) => {
      log::error!("invalid authorization failed to check header: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  };
  let kid = match header
    .jwk
    .as_ref()
    .and_then(|jwk| jwk.common.key_id.as_ref())
  {
    Some(kid) => kid.to_owned(),
    None => {
      log::error!("invalid authorization kid is missing");
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  };
  let alg = header.alg.clone();
  let jwk_sql_row = match get_jwk_by_kid(pool, kid).await {
    Ok(Some(jwk)) => jwk,
    Ok(None) => {
      log::error!("invalid JWK not found by kid");
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
    Err(e) => {
      log::error!("invalid authorization token is invalid: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  };
  let jwk = match jwk_sql_row.clone().try_into() {
    Ok(jwk) => jwk,
    Err(e) => {
      log::error!("invalid JWK unable to convert to token decoding key: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  };
  let decoding_key = match DecodingKey::from_jwk(&jwk) {
    Ok(decoding_key) => decoding_key,
    Err(e) => {
      log::error!(
        "invalid JWK could not convert to a token decoding key: {}",
        e
      );
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  };
  let token_data = match parse_jwt::<T>(authorization_string, app_config, decoding_key, alg) {
    Ok(token_data) => token_data,
    Err(e) => {
      log::error!("invalid authorization failed to parse claims: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  };

  // Check if token is revoked
  match is_token_revoked(pool, authorization_string).await {
    Ok(true) => {
      log::error!("token has been revoked");
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
    Ok(false) => {
      // Token is not revoked, continue
    }
    Err(e) => {
      log::error!("failed to check token revocation status: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  }

  Ok((token_data, jwk_sql_row))
}
