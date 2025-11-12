use axum::{extract::State, response::IntoResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  current_user::{constants::TAG, entity::User},
  entity::RouterState,
  error::HttpError,
  middleware::user_authorization::UserAuthorization,
};

#[utoipa::path(
  get,
  path = "/current-user",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = User),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn current_user(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
) -> impl IntoResponse {
  match user_authorization.get_user(&state.pool).await {
    Ok(user) => axum::Json(user).into_response(),
    Err(e) => return e.into_response(),
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(current_user))
    .with_state(state)
}
