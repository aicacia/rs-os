use axum::{extract::State, response::IntoResponse};
use os_api::HttpError;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  discovery::{constants::TAG, entity::ServicesDiscovered},
  entity::RouterState,
};

#[utoipa::path(
  get,
  path = "/.well-known/services",
  tags = [TAG],
  responses(
    (status = 200, description = "Services Discovered", body = ServicesDiscovered),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn service_discovery(State(state): State<RouterState>) -> impl IntoResponse {
  axum::Json(ServicesDiscovered::from(&state.config.services)).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(service_discovery))
    .with_state(state)
}
