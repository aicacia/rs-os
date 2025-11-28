use axum::{extract::State, response::IntoResponse};
use http::StatusCode;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::jwk::sql::get_jwk_for_sign_and_verify,
  model::user::sql::create_user_with_password,
  router::{
    common::{constants::TOKEN_ISSUE_TYPE_PASSWORD, entity::Token, helper::create_user_token},
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR},
    Json,
    register::{constants::TAG, entity::SignupRequest},
  },
};

#[utoipa::path(
  post,
  path = "/register",
  tags = [TAG],
  request_body(content = SignupRequest, content_type = "application/json"),
  responses(
    (status = 201, description = "Token created", body = Token),
    (status = 401, description = "Invalid username or password", body = HttpError),
    (status = 403, description = "Password sign in not allowed", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn register(
  State(state): State<RouterState>,
  Json(register_request): Json<SignupRequest>,
) -> impl IntoResponse {
  let user = match create_user_with_password(
    &state.pool,
    &state.config,
    &register_request.username,
    &register_request.password,
  )
  .await
  {
    Ok(user) => user,
    Err(e) => {
      log::error!("error creating user: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let jwk = match get_jwk_for_sign_and_verify(&state.pool).await {
    Ok(Some(jwk)) => jwk,
    Ok(None) => {
      log::error!("error no valid jwk for signing and verifying jwts");
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("error getting jwk: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  match create_user_token(
    &state.pool,
    &state.config,
    jwk,
    user,
    state.config.api_url(),
    register_request
      .scope
      .unwrap_or_else(|| "openid".to_owned()),
    TOKEN_ISSUE_TYPE_PASSWORD.to_owned(),
  )
  .await
  {
    Ok(token) => (StatusCode::CREATED, axum::Json(token)).into_response(),
    Err(e) => e.into_response(),
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(register))
    .with_state(state)
}
