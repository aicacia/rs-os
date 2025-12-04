use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  model::user::sql::{
    delete_user_oauth2_provider, get_user_oauth2_provider_by_id, get_user_oauth2_providers,
    link_user_oauth2_provider,
  },
  router::{
    common::permissions::Permission,
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, NOT_FOUND_ERROR},
    middleware::user_authorization::UserAuthorization,
    user_oauth2_provider::{
      constants::TAG,
      entity::{LinkUserOAuth2ProviderRequest, UserOAuth2Provider},
    },
  },
};

#[utoipa::path(
  get,
  path = "/users/{user_id}/oauth2-providers",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = [UserOAuth2Provider]),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn list_user_oauth2_providers(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(user_id): Path<String>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  // Users can read their own OAuth2 providers, admins can read any
  if user_authorization.user_sql_row.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match get_user_oauth2_providers(&state.pool, user_id_parsed).await {
    Ok(providers) => {
      let providers: Vec<UserOAuth2Provider> = providers.into_iter().map(|p| p.into()).collect();
      axum::Json(providers).into_response()
    }
    Err(e) => {
      log::error!("error listing user oauth2 providers: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  get,
  path = "/users/{user_id}/oauth2-providers/{provider_id}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = UserOAuth2Provider),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn get_user_oauth2_provider(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path((user_id, provider_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let provider_id_parsed = match provider_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("provider_id", "invalid_id")
        .into_response();
    }
  };

  // Users can read their own OAuth2 providers, admins can read any
  if user_authorization.user_sql_row.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  let provider_sql_row =
    match get_user_oauth2_provider_by_id(&state.pool, user_id_parsed, provider_id_parsed).await {
      Ok(Some(provider)) => provider,
      Ok(None) => {
        return HttpError::not_found()
          .with_error("oauth2_provider", NOT_FOUND_ERROR)
          .into_response();
      }
      Err(e) => {
        log::error!("error fetching user oauth2 provider: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let provider: UserOAuth2Provider = provider_sql_row.into();
  axum::Json(provider).into_response()
}

#[utoipa::path(
  post,
  path = "/users/{user_id}/oauth2-providers",
  tags = [TAG],
  request_body = LinkUserOAuth2ProviderRequest,
  responses(
    (status = 201, content_type = "application/json", body = UserOAuth2Provider),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn link_user_oauth2_provider_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(user_id): Path<String>,
  Json(request): Json<LinkUserOAuth2ProviderRequest>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  // Users can link their own OAuth2 providers, admins can link any
  if user_authorization.user_sql_row.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserWrite) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  let provider_sql_row = match link_user_oauth2_provider(
    &state.pool,
    user_id_parsed,
    request.provider_id,
    &request.name,
    &request.email,
  )
  .await
  {
    Ok(provider) => provider,
    Err(e) => {
      log::error!("error linking user oauth2 provider: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let provider: UserOAuth2Provider = provider_sql_row.into();
  (axum::http::StatusCode::CREATED, axum::Json(provider)).into_response()
}

#[utoipa::path(
  delete,
  path = "/users/{user_id}/oauth2-providers/{provider_id}",
  tags = [TAG],
  responses(
    (status = 204, description = "OAuth2 provider unlinked successfully"),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn unlink_user_oauth2_provider_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path((user_id, provider_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let provider_id_parsed = match provider_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("provider_id", "invalid_id")
        .into_response();
    }
  };

  // Users can unlink their own OAuth2 providers, admins can unlink any
  if user_authorization.user_sql_row.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserWrite) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match delete_user_oauth2_provider(&state.pool, user_id_parsed, provider_id_parsed).await {
    Ok(Some(_)) => axum::http::StatusCode::NO_CONTENT.into_response(),
    Ok(None) => HttpError::not_found()
      .with_error("oauth2_provider", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error unlinking user oauth2 provider: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(list_user_oauth2_providers))
    .routes(routes!(get_user_oauth2_provider))
    .routes(routes!(link_user_oauth2_provider_handler))
    .routes(routes!(unlink_user_oauth2_provider_handler))
    .with_state(state)
}
