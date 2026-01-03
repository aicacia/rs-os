use axum::{
  extract::{Query, State},
  response::IntoResponse,
};

use os_api::{Authorization, BasicClaims, error::HttpError};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  fs::{constants::TAG, entity::ListQuery},
};

#[utoipa::path(
  get,
  path = "/",
  tags = [TAG],
  params(ListQuery),
  responses(
    (status = 204, content_type = "application/json"),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["fs:read"])
  )
)]
pub async fn list(
  State(state): State<RouterState>,
  user_authorization: Authorization<BasicClaims>,
  Query(ListQuery { prefix }): Query<ListQuery>,
) -> impl IntoResponse {
  axum::Json(()).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new().routes(routes!(list)).with_state(state)
}
