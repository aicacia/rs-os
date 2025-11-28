use axum::extract::{FromRef, FromRequestParts};
use http::{HeaderValue, request::Parts};

use crate::router::{
  entity::RouterState,
  error::{HttpError, INVALID_ERROR, NOT_SUPPORTED_ERROR, REQUIRED_ERROR},
  middleware::constants::{AUTHORIZATION_BEARER_PREFIX, AUTHORIZATION_HEADER},
};

pub struct Authorization<T>
where
  T: Send + Sync,
{
  pub claims: T,
}

impl<S, T> FromRequestParts<S> for Authorization<T>
where
  RouterState: FromRef<S>,
  S: Send + Sync,
  T: Send + Sync,
{
  type Rejection = HttpError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let router_state = RouterState::from_ref(state);

    if let Some(authorization_header_value) = parts.headers.get(AUTHORIZATION_HEADER) {
      let authorization_string = authorization_from_header(authorization_header_value)?;

      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, NOT_SUPPORTED_ERROR));
    }
    Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR))
  }
}

pub fn authorization_from_header(
  authorization_header_value: &HeaderValue,
) -> Result<&str, HttpError> {
  match authorization_header_value.to_str() {
    Ok(authorization_string) => {
      if authorization_string.len() < AUTHORIZATION_BEARER_PREFIX.len() {
        log::error!("invalid authorization header is invalid");
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      Ok(&authorization_string[(AUTHORIZATION_BEARER_PREFIX.len())..])
    }
    Err(e) => {
      log::error!("invalid authorization header is invalid: {}", e);
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
    }
  }
}
