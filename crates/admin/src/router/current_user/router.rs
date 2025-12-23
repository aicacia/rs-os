use axum::{extract::State, response::IntoResponse};
use http::StatusCode;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::encryption::encrypt_password,
  router::{
    Json,
    current_user::{
      constants::TAG,
      entity::{
        CurrentUser, UpdateUserInfoRequest, UpdateUserPassword, UpdateUsernameRequest, UserInfo,
      },
    },
    entity::RouterState,
    error::HttpError,
    middleware::user_authorization::UserAuthorization,
  },
};
use os_model::entities::{
  user_infos::update_user_info as update_user_info_orm,
  users::{update_user, update_user_password},
};

#[utoipa::path(
  get,
  path = "/current-user",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = CurrentUser),
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
  match user_authorization
    .get_user(&state.database_connection)
    .await
  {
    Ok(user) => axum::Json(user).into_response(),
    Err(e) => return e.into_response(),
  }
}

#[utoipa::path(
  patch,
  path = "/current-user",
  tags = [TAG],
  request_body(content = UpdateUsernameRequest, content_type = "application/json"),
  responses(
    (status = 204, content_type = "application/json"),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn update_username(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(update): Json<UpdateUsernameRequest>,
) -> impl IntoResponse {
  match update_user(
    &state.database_connection,
    user_authorization.user_model.id,
    &update.username,
  )
  .await
  {
    Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
    Err(e) => {
      log::error!("error updating username: {}", e);
      return HttpError::internal_error().into_response();
    }
  }
}

#[utoipa::path(
  patch,
  path = "/current-user/password",
  tags = [TAG],
  request_body(content = UpdateUserPassword, content_type = "application/json"),
  responses(
    (status = 204, content_type = "application/json"),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn update_password(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(update): Json<UpdateUserPassword>,
) -> impl IntoResponse {
  let app_config = state.config.clone();
  match update_user_password(
    &state.database_connection,
    user_authorization.user_model.id,
    update.password.as_str(),
    |password| encrypt_password(&app_config, password).map_err(|e| e.into()),
  )
  .await
  {
    Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
    Err(e) => {
      log::error!("error updating user password: {}", e);
      return HttpError::internal_error().into_response();
    }
  }
}

#[utoipa::path(
  patch,
  path = "/current-user/info",
  tags = [TAG],
  request_body(content = UpdateUserInfoRequest, content_type = "application/json"),
  responses(
    (status = 200, content_type = "application/json", body = UserInfo),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn update_user_info(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(update): Json<UpdateUserInfoRequest>,
) -> impl IntoResponse {
  match update_user_info_orm(
    &state.database_connection,
    user_authorization.user_model.id,
    update.into(),
  )
  .await
  {
    Ok(user_info) => axum::Json(Into::<UserInfo>::into(user_info)).into_response(),
    Err(e) => {
      log::error!("error updating user info: {}", e);
      return HttpError::internal_error().into_response();
    }
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(current_user))
    .routes(routes!(update_username))
    .routes(routes!(update_password))
    .routes(routes!(update_user_info))
    .with_state(state)
}
