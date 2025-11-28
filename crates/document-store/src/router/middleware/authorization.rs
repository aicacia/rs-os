use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use os_api::{HttpError, NOT_SUPPORTED_ERROR, REQUIRED_ERROR, authorization_from_header, AUTHORIZATION_HEADER};

use crate::router::entity::RouterState;

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
    let _router_state = RouterState::from_ref(state);

    if let Some(authorization_header_value) = parts.headers.get(AUTHORIZATION_HEADER) {
      let _authorization_string = authorization_from_header(authorization_header_value)?;

      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, NOT_SUPPORTED_ERROR));
    }
    Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR))
  }
}
