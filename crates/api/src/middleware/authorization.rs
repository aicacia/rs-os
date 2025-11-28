use axum::http::HeaderValue;

use crate::error::{HttpError, INVALID_ERROR};

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
