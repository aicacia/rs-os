use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::helper::json_to_string_vec,
  model::{
    client::sql::{deactivate_client, get_client_by_client_id, list_clients, upsert_client},
    user::orm::{get_user_client_by_client_id, upsert_user_client},
  },
  router::{
    client::{
      constants::TAG,
      entity::{Client, ClientAllowed, ClientAuthorization, ClientAuthorizeRequest},
    },
    common::{helper::create_user_authorization_code_token, permissions::Permission},
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_ALLOWED_ERROR, NOT_FOUND_ERROR},
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
  match user_authorization.has_permission(Permission::ClientRead) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let client_sql_row = match get_client_by_client_id(&state.database, &client_id).await {
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
  request_body(content = crate::router::oidc::entity::ClientRegisterRequest, content_type = "application/json"),
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
  Json(client_register_request): Json<crate::router::oidc::entity::ClientRegisterRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let (client_sql_row, is_new) =
    match upsert_client(&state.database, client_register_request.into()).await {
      Ok(r) => r,
      Err(e) => {
        log::error!("error upserting client: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let client: Client = client_sql_row.into();
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
  request_body(content = crate::router::oidc::entity::ClientRegisterRequest, content_type = "application/json"),
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
  Json(client_register_request): Json<crate::router::oidc::entity::ClientRegisterRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  if client_register_request.client_id != client_id {
    return HttpError::bad_request()
      .with_error("client_id", INVALID_ERROR)
      .into_response();
  }

  let (client_sql_row, _is_new) =
    match upsert_client(&state.database, client_register_request.into()).await {
      Ok(r) => r,
      Err(e) => {
        log::error!("error updating client: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let client: Client = client_sql_row.into();
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
    Ok(Some(_client_sql_row)) => axum::http::StatusCode::NO_CONTENT.into_response(),
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
  match get_user_client_by_client_id(&state.database, user_authorization.user_model.id, &client_id)
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
  let client_sql_row = match get_client_by_client_id(&state.database, &client_id).await {
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

  let scopes = json_to_string_vec(&client_sql_row.scopes);
  match upsert_user_client(
    &state.database,
    user_authorization.user_model.id,
    &client_id,
    scopes,
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

#[utoipa::path(
  post,
  path = "/client/{client_id}/authorize",
  tags = [TAG],
  request_body(content = ClientAuthorizeRequest, content_type = "application/json"),
  responses(
    (status = 200, description = "Authorized", body = ClientAuthorization),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn client_authorize(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
  Json(authorization_request): Json<ClientAuthorizeRequest>,
) -> impl IntoResponse {
  let client_sql_row = match get_client_by_client_id(&state.database, &client_id).await {
    Ok(Some(client_sql_row)) => client_sql_row,
    Ok(None) => {
      return HttpError::not_found()
        .with_error("client", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("failed to fetch client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  match get_user_client_by_client_id(
    &state.database,
    user_authorization.user_model.id,
    &client_sql_row.client_id,
  )
  .await
  {
    Ok(Some(_client_allowed)) => {}
    Ok(None) => {
      return HttpError::forbidden()
        .with_error("client", NOT_ALLOWED_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("failed to check user client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  }

  let redirect_uri = match Url::parse(&authorization_request.redirect_uri) {
    Ok(redirect_uri) => redirect_uri,
    Err(e) => {
      log::error!("invalid redirect_uri: {}", e);
      return HttpError::bad_request()
        .with_error("redirect_uri", INVALID_ERROR)
        .into_response();
    }
  };

  if let Some(redirect_uris) = client_sql_row
    .redirect_uris
    .as_ref()
    .map(json_to_string_vec)
  {
    let redirect_uri_string = redirect_uri.origin().ascii_serialization() + redirect_uri.path();
    if !redirect_uris.contains(&redirect_uri_string) {
      return HttpError::bad_request()
        .with_error("redirect_uri", NOT_ALLOWED_ERROR)
        .into_response();
    }
  } else {
    return HttpError::bad_request()
      .with_error("client", INVALID_ERROR)
      .into_response();
  }

  let authorization_code_token = if authorization_request.response_type.needs_code() {
    let (code_challenge, code_challenge_method) = match (
      &authorization_request.code_challenge,
      &authorization_request.code_challenge_method,
    ) {
      (Some(challenge), Some(method)) => (challenge.clone(), method.clone()),
      _ => {
        return HttpError::bad_request()
          .with_error("code_challenge", INVALID_ERROR)
          .into_response();
      }
    };

    match create_user_authorization_code_token(
      &state.database,
      &state.config,
      user_authorization.user_model.id,
      client_id,
      authorization_request.scope,
      code_challenge,
      code_challenge_method,
    )
    .await
    {
      Ok(code) => Some(code),
      Err(e) => {
        return e.into_response();
      }
    }
  } else {
    None
  };

  if let Some(authorization_code_token) = authorization_code_token {
    axum::Json(ClientAuthorization::AuthorizationCode {
      code: authorization_code_token,
    })
    .into_response()
  } else {
    HttpError::bad_request()
      .with_error("response_type", INVALID_ERROR)
      .into_response()
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(client_by_client_id))
    .routes(routes!(client_list))
    .routes(routes!(client_create))
    .routes(routes!(client_update))
    .routes(routes!(client_delete))
    .routes(routes!(client_authorize))
    .routes(routes!(client_user_allowed))
    .routes(routes!(client_user_approve))
    .with_state(state)
}
