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
    client::sql::get_client_by_client_id,
    user::sql::{get_user_client_by_client_id, upsert_user_client},
  },
  router::{
    client::{
      constants::{CLIENT_READ, TAG},
      entity::{Client, ClientAllowed, ClientAuthorization, ClientAuthorizeRequest},
    },
    common::helper::create_user_auhorization_code_token,
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_ALLOWED_ERROR, NOT_FOUND_ERROR},
    middleware::user_authorization::UserAuthorization,
    oidc::{entity::ResponseType, router::get_audiences_by_client},
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

#[utoipa::path(
  post,
  path = "/client/{client_id}/authorize",
  tags = [TAG],
  request_body(content = ClientAuthorizeRequest, content_type = "application/json"),
  responses(
    (status = 200, description = "Authorized", body = ClientAuthorization),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Application Error", body = HttpError),
    (status = 403, description = "Application Error", body = HttpError),
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
  let client_sql_row = match get_client_by_client_id(&state.pool, &client_id).await {
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
    &state.pool,
    user_authorization.user_sql_row.id,
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

  let audiences = match get_audiences_by_client(&client_sql_row) {
    Ok(audiences) => audiences,
    Err(e) => return e.into_response(),
  };

  let authorization_response = match authorization_request.response_type {
    ResponseType::None => todo!(),
    ResponseType::Code => match create_user_auhorization_code_token(
      &state.pool,
      &state.config,
      user_authorization.user_sql_row,
      &audiences,
    )
    .await
    {
      Ok(code) => ClientAuthorization::AuthorizationCode { code },
      Err(e) => {
        return e.into_response();
      }
    },
    ResponseType::Token => todo!(),
    ResponseType::IdToken => todo!(),
    ResponseType::CodeToken => todo!(),
    ResponseType::CodeIdToken => todo!(),
    ResponseType::IdTokenToken => todo!(),
    ResponseType::CodeIdTokenToken => todo!(),
  };

  axum::Json(authorization_response).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(client_by_client_id))
    .routes(routes!(client_authorize))
    .routes(routes!(client_user_allowed))
    .routes(routes!(client_user_approve))
    .with_state(state)
}
