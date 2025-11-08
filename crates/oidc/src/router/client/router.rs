use axum::{
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  model::client::sql::get_client_by_client_id,
  router::{
    client::{
      constants::{CLIENT_CREATE, TAG},
      entity::Client,
    },
    common::entity::Claims,
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, NOT_FOUND_ERROR},
    middleware::user_authorization::UserAuthorization,
  },
};

#[utoipa::path(
  get,
  path = "/clients/{client_id}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = Client),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:read"])
  )
)]
pub async fn client_by_client_id(
  State(state): State<RouterState>,
  UserAuthorization { .. }: UserAuthorization,
  Path(client_id): Path<String>,
) -> impl IntoResponse {
  let client_sql_row = match get_client_by_client_id(&state.pool, &client_id).await {
    Ok(Some(client)) => client,
    Ok(None) => {
      return HttpError::not_found()
        .with_error("client_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("error fetching user emails: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let client: Client = client_sql_row.into();

  axum::Json(client).into_response()
}

#[utoipa::path(
  post,
  path = "/clients",
  tags = [TAG],
  responses(
    (status = 201, content_type = "application/json", body = Client),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:create"])
  )
)]
pub async fn create_client(
  State(state): State<RouterState>,
  UserAuthorization { claims, .. }: UserAuthorization,
) -> impl IntoResponse {
  if !claims.has_scope(CLIENT_CREATE) {
    return HttpError::forbidden()
      .with_error("scopes", CLIENT_CREATE)
      .into_response();
  }

  axum::Json(()).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(client_by_client_id))
    .routes(routes!(create_client))
    .with_state(state)
}
