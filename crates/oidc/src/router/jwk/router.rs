use axum::{
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::jwk::orm::get_jwk_by_kid,
  router::{
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, NOT_FOUND_ERROR},
    jwk::{constants::TAG, entity::JWK},
  },
};

#[utoipa::path(
  get,
  path = "/jwks/{kid}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = JWK),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  )
)]
pub async fn jwk_by_id(
  State(state): State<RouterState>,
  Path(kid): Path<i64>,
) -> impl IntoResponse {
  let jwk_sql_row = match get_jwk_by_kid(&state.database, kid.to_string()).await {
    Ok(Some(jwk)) => jwk,
    Ok(None) => {
      log::error!("invalid JWK not found by kid");
      return HttpError::not_found()
        .with_error("kid", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("invalid authorization token is invalid: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let jwk = JWK {
    key_ops: jwk_sql_row.public_key_operations(),

    kid: jwk_sql_row.kid.to_string(),
    kty: jwk_sql_row.kty,
    alg: jwk_sql_row.alg,
    r#use: jwk_sql_row.r#use,

    n: jwk_sql_row.n,
    e: jwk_sql_row.e,

    crv: jwk_sql_row.crv,
    x: jwk_sql_row.x,
    y: jwk_sql_row.y,

    x5c: jwk_sql_row.x5c,
    x5u: jwk_sql_row.x5u,
    x5t: jwk_sql_row.x5t,
    x5t_s256: jwk_sql_row.x5t_s256,
  };

  axum::Json(jwk).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(jwk_by_id))
    .with_state(state)
}
