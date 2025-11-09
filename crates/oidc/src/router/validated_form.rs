use axum::{
  Form,
  extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::router::error::{HttpError, REQUEST_BODY};

// TODO: use form request forms
pub struct ValidatedForm<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedForm<T>
where
  S: Send + Sync,
  T: DeserializeOwned + Validate + Send + 'static,
{
  type Rejection = HttpError;

  async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Form(value) = match Form::<T>::from_request(request, state).await {
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
