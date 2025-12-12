use axum::{extract::State, http::StatusCode, response::IntoResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  util::{
    constants::TAG,
    entity::{Health, Version},
  },
};

#[utoipa::path(
  get,
  path = "/health",
  tags = [TAG],
  responses(
    (status = 200, description = "Health check response", body = Health),
    (status = 500, description = "Health check response", body = Health),
  )
)]
pub async fn health(State(_state): State<RouterState>) -> impl IntoResponse {
  // TODO: Implement proper health check by running a simple query
  let health = Health {
    db: true,
  };

  let status = if health.is_healthy() {
    StatusCode::OK
  } else {
    StatusCode::INTERNAL_SERVER_ERROR
  };

  (status, axum::Json(health))
}

#[utoipa::path(
  get,
  path = "/version",
  tags = [TAG],
  responses(
    (status = 200, description = "Version response", body = Version),
  )
)]
pub async fn version() -> axum::Json<Version> {
  axum::Json(Version::default())
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(health))
    .routes(routes!(version))
    .with_state(state)
}
