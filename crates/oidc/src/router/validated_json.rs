use axum::{
  Json,
  extract::{FromRequest, Request, rejection::JsonRejection},
};
use validator::Validate;

use crate::router::error::{HttpError, REQUEST_BODY};

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
  T: Validate,
  Json<T>: FromRequest<S, Rejection = JsonRejection>,
  S: Send + Sync,
{
  type Rejection = HttpError;

  async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = match Json::<T>::from_request(request, state).await {
      Ok(value) => value,
      Err(rejection) => {
        return Err(HttpError::bad_request().with_error(REQUEST_BODY, rejection.to_string()));
      }
    };

    match value.validate() {
      Ok(_) => (),
      Err(errors) => return Err(HttpError::from(errors)),
    };

    Ok(Self(value))
  }
}
