use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use os_model::entities::user_o_auth2_providers::{
  delete_user_oauth2_provider, get_user_oauth2_provider_by_id, get_user_oauth2_providers,
  link_user_oauth2_provider,
};
use crate::{
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
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match get_user_oauth2_providers(&state.database, user_id_parsed).await {
    Ok(providers) => {
      let providers: Vec<UserOAuth2Provider> = providers
        .into_iter()
        .filter_map(|(provider, provider_info_opt)| {
          provider_info_opt.map(|provider_info| UserOAuth2Provider {
            oauth2_provider_id: provider_info.id.to_string(),
            user_id: provider.user_id.to_string(),
            uri: provider_info.uri,
            name: provider_info.description,
            email: provider.email,
            updated_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.updated_at, 0)
              .unwrap_or_default(),
            created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.created_at, 0)
              .unwrap_or_default(),
          })
        })
        .collect();
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
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match get_user_oauth2_provider_by_id(&state.database, user_id_parsed, provider_id_parsed).await {
    Ok(Some((provider, provider_info_opt))) => {
      if let Some(provider_info) = provider_info_opt {
        let oauth2_provider = UserOAuth2Provider {
          oauth2_provider_id: provider_info.id.to_string(),
          user_id: provider.user_id.to_string(),
          uri: provider_info.uri,
          name: provider_info.description,
          email: provider.email,
          updated_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.updated_at, 0)
            .unwrap_or_default(),
          created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.created_at, 0)
            .unwrap_or_default(),
        };
        axum::Json(oauth2_provider).into_response()
      } else {
        HttpError::not_found()
          .with_error("oauth2_provider", NOT_FOUND_ERROR)
          .into_response()
      }
    }
    Ok(None) => HttpError::not_found()
      .with_error("oauth2_provider", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error fetching user oauth2 provider: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
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
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserWrite) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match link_user_oauth2_provider(
    &state.database,
    user_id_parsed,
    request.provider_id,
    &request.name,
    &request.email,
  )
  .await
  {
    Ok(provider) => {
      let oauth2_provider = UserOAuth2Provider {
        oauth2_provider_id: request.provider_id.to_string(),
        user_id: user_id_parsed.to_string(),
        uri: String::new(), // TODO: Need to fetch provider info for URI
        name: request.name,
        email: request.email,
        updated_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.updated_at, 0)
          .unwrap_or_default(),
        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.created_at, 0)
          .unwrap_or_default(),
      };
      (axum::http::StatusCode::CREATED, axum::Json(oauth2_provider)).into_response()
    }
    Err(e) => {
      log::error!("error linking user oauth2 provider: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
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
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserWrite) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match delete_user_oauth2_provider(&state.database, user_id_parsed, provider_id_parsed).await {
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
