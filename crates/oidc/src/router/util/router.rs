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
pub async fn health(State(state): State<RouterState>) -> impl IntoResponse {
  let health = Health {
    db: match state.database_connection.ping().await {
      Ok(()) => true,
      Err(e) => {
        log::error!("Database health check failed: {}", e);
        false
      }
    },
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
