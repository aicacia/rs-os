use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use jsonwebtoken::DecodingKey;
use os_api::{
  AUTHORIZATION_HEADER, HttpError, INVALID_ERROR, REQUIRED_ERROR, authorization_from_header,
};

use crate::{
  core::config::AppConfig,
  router::{
    common::{entity::Claims, helper::parse_jwt},
    entity::RouterState,
  },
};
use os_model::entities::{
  jwks::{get_jwk_by_kid, model_to_jwt_jwk},
  revoked_tokens::is_token_revoked,
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
      let (token_data, _jwk_model) = parse_authorization(
        &router_state.database,
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

pub async fn parse_authorization<T>(
  db: &sea_orm::DatabaseConnection,
  app_config: &AppConfig,
  authorization_string: &str,
) -> Result<(jsonwebtoken::TokenData<T>, os_model::entities::jwks::Model), HttpError>
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
  let jwk_row = match get_jwk_by_kid(db, kid).await {
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
  let jwk = match model_to_jwt_jwk(jwk_row.clone()) {
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

  match is_token_revoked(db, authorization_string).await {
    Ok(true) => {
      log::error!("token has been revoked");
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
    Ok(false) => {}
    Err(e) => {
      log::error!("failed to check token revocation status: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  }

  Ok((token_data, jwk_row))
}
