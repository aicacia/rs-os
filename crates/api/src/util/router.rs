use axum::{extract::State, http::StatusCode, response::IntoResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  state::RouterState,
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
pub async fn health<C>(State(_state): State<RouterState<C>>) -> impl IntoResponse {
  (StatusCode::OK, axum::Json(Health { ok: true }))
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

pub fn create_router<C>(state: RouterState<C>) -> OpenApiRouter
where
  C: Clone + Send + Sync + 'static,
{
  OpenApiRouter::new()
    .routes(routes!(health))
    .routes(routes!(version))
    .with_state(state)
}
