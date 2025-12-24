use axum::{extract::State, http::StatusCode, response::IntoResponse};
use core::future::Future;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::util::{
  constants::TAG,
  entity::{Health, Version},
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
pub async fn health<S, F, Fut>(State((state, health_check)): State<(S, F)>) -> impl IntoResponse
where
  S: Clone + Send + Sync + 'static,
  F: Fn(S) -> Fut + Clone + Send + Sync + 'static,
  Fut: Future<Output = bool> + Send,
{
  let ok: bool = health_check(state).await;
  let status = if ok {
    StatusCode::OK
  } else {
    StatusCode::INTERNAL_SERVER_ERROR
  };

  (status, axum::Json(Health { ok })).into_response()
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

pub fn create_router<S, F, Fut>(state: S, health_check: F) -> OpenApiRouter
where
  S: Clone + Send + Sync + 'static,
  F: Fn(S) -> Fut + Clone + Send + Sync + 'static,
  Fut: Future<Output = bool> + Send + 'static,
{
  OpenApiRouter::new()
    .routes(routes!(health))
    .with_state((state, health_check))
    .routes(routes!(version))
}
