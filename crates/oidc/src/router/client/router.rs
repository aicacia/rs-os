use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::helper::json_to_string_vec,
  model::{
    client::sql::{get_client_by_client_id, upsert_client},
    user::sql::{get_user_client_by_client_id, upsert_user_client},
  },
  router::{
    client::{
      constants::{CLIENT_CREATE, CLIENT_READ, TAG},
      entity::{Client, ClientAllowed, ClientUpsertRequest},
    },
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, NOT_ALLOWED_ERROR, NOT_FOUND_ERROR},
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
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:read"])
  )
)]
pub async fn client_by_client_id(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
) -> impl IntoResponse {
  match user_authorization.has_permission(CLIENT_READ) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let client_sql_row = match get_client_by_client_id(&state.pool, &client_id).await {
    Ok(Some(client)) => client,
    Ok(None) => {
      return HttpError::not_found()
        .with_error("client", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("error fetching client: {}", e);
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
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:create"])
  )
)]
pub async fn create_client(
  State(_state): State<RouterState>,
  user_authorization: UserAuthorization,
) -> impl IntoResponse {
  match user_authorization.has_permission(CLIENT_CREATE) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  axum::Json(()).into_response()
}

#[utoipa::path(
  post,
  path = "/clients:upsert",
  tags = [TAG],
  request_body(content = ClientUpsertRequest, content_type = "application/json"),
  responses(
    (status = 200, content_type = "application/json", body = Client),
    (status = 201, content_type = "application/json", body = Client),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:create"])
  )
)]
pub async fn client_upsert(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(client_upsert_request): Json<ClientUpsertRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(CLIENT_CREATE) {
    Ok(_) => {}
    Err(e) => {
      log::error!("error fetching client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let client_sql_row = match upsert_client(&state.pool, client_upsert_request.into()).await {
    Ok(client_sql_row) => client_sql_row,
    Err(e) => {
      log::error!("error upserting client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let client: Client = client_sql_row.into();

  axum::Json(client).into_response()
}

#[utoipa::path(
  get,
  path = "/clients/{client_id}/allowed",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = ClientAllowed),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn client_user_allowed(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
) -> impl IntoResponse {
  match get_user_client_by_client_id(&state.pool, user_authorization.user_sql_row.id, &client_id)
    .await
  {
    Ok(Some(user_client_sql_row)) => axum::Json(ClientAllowed {
      allowed_scopes: json_to_string_vec(user_client_sql_row.allowed_scopes),
    })
    .into_response(),
    Ok(None) => HttpError::forbidden()
      .with_error("client", NOT_ALLOWED_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error fetching user client: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/clients/{client_id}/approve",
  tags = [TAG],
  responses(
    (status = 204, content_type = "application/json"),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn client_user_approve(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
) -> impl IntoResponse {
  let client_sql_row = match get_client_by_client_id(&state.pool, &client_id).await {
    Ok(Some(client)) => client,
    Ok(None) => {
      return HttpError::not_found()
        .with_error("client", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("error fetching client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  match upsert_user_client(
    &state.pool,
    user_authorization.user_sql_row.id,
    client_id,
    client_sql_row.scopes,
  )
  .await
  {
    Ok(_user_client) => {}
    Err(e) => {
      log::error!("error approving client for user: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  }

  axum::Json(()).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(client_by_client_id))
    .routes(routes!(create_client))
    .routes(routes!(client_upsert))
    .routes(routes!(client_user_allowed))
    .routes(routes!(client_user_approve))
    .with_state(state)
}
