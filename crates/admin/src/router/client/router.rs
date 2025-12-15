use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  client::{
    constants::TAG,
    entity::{Client, ClientUpsertRequest, client_upsert_request_changed},
  },
  common::permissions::Permission,
  entity::RouterState,
  error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_FOUND_ERROR},
  middleware::user_authorization::UserAuthorization,
};
use os_model::entities::clients::{
  deactivate_client, get_client_by_client_id, list_clients, upsert_client,
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
  match user_authorization.has_permission(Permission::ClientRead) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let client_model = match get_client_by_client_id(&state.database, &client_id).await {
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

  let client: Client = client_model.into();

  axum::Json(client).into_response()
}

#[utoipa::path(
  get,
  path = "/clients",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = [Client]),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:read"])
  )
)]
pub async fn client_list(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientRead) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  match list_clients(&state.database).await {
    Ok(clients) => {
      let clients: Vec<Client> = clients.into_iter().map(|c| c.into()).collect();
      axum::Json(clients).into_response()
    }
    Err(e) => {
      log::error!("error listing clients: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/clients",
  tags = [TAG],
  request_body(content = ClientUpsertRequest, content_type = "application/json"),
  responses(
    (status = 200, description = "Client updated", body = Client),
    (status = 201, description = "Client created", body = Client),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:write"])
  )
)]
pub async fn client_create(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(client_upsert_request): Json<ClientUpsertRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let (client_model, is_new) = match upsert_client(
    &state.database,
    client_upsert_request.into(),
    crate::core::encryption::random_bytes,
  )
  .await
  {
    Ok(r) => r,
    Err(e) => {
      log::error!("error upserting client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let client: Client = client_model.into();
  if is_new {
    (axum::http::StatusCode::CREATED, axum::Json(client)).into_response()
  } else {
    axum::Json(client).into_response()
  }
}

#[utoipa::path(
  put,
  path = "/clients/{client_id}",
  tags = [TAG],
  request_body(content = ClientUpsertRequest, content_type = "application/json"),
  responses(
    (status = 200, description = "Client updated", body = Client),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 404, description = "Not Found", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:write"])
  )
)]
pub async fn client_update(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
  Json(client_upsert_request): Json<ClientUpsertRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  if client_upsert_request.client_id != client_id {
    return HttpError::bad_request()
      .with_error("client_id", INVALID_ERROR)
      .into_response();
  }

  let existing_client = match get_client_by_client_id(&state.database, &client_id).await {
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

  if !client_upsert_request_changed(&client_upsert_request, &existing_client) {
    let client: Client = existing_client.into();
    return axum::Json(client).into_response();
  }

  let (client_model, _is_new) = match upsert_client(
    &state.database,
    client_upsert_request.into(),
    crate::core::encryption::random_bytes,
  )
  .await
  {
    Ok(r) => r,
    Err(e) => {
      log::error!("error updating client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let client: Client = client_model.into();
  axum::Json(client).into_response()
}

#[utoipa::path(
  delete,
  path = "/clients/{client_id}",
  tags = [TAG],
  responses(
    (status = 204, description = "Client deleted"),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 404, description = "Not Found", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:delete"])
  )
)]
pub async fn client_delete(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientDelete) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  match deactivate_client(&state.database, &client_id).await {
    Ok(Some(_client_model)) => axum::http::StatusCode::NO_CONTENT.into_response(),
    Ok(None) => HttpError::not_found()
      .with_error("client", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error deleting client: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(client_by_client_id))
    .routes(routes!(client_list))
    .routes(routes!(client_create))
    .routes(routes!(client_update))
    .routes(routes!(client_delete))
    .with_state(state)
}
