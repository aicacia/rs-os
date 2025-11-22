use crate::router::json::Json;
use axum::{extract::State, response::IntoResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  current_user::{constants::TAG, entity::User},
  entity::RouterState,
  error::HttpError,
  middleware::user_authorization::UserAuthorization,
};

use crate::model::user::sql::{update_user_password, update_user_username};
use crate::router::current_user::entity::UpdateUserInfoRequest;
use crate::router::current_user::entity::{UpdateUserPassword, UpdateUsernameRequest};

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

#[utoipa::path(
  patch,
  path = "/current-user",
  tags = [TAG],
  request_body(content = UpdateUsernameRequest, content_type = "application/json"),
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
pub async fn update_username(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(update): Json<UpdateUsernameRequest>,
) -> impl IntoResponse {
  match update_user_username(
    &state.pool,
    user_authorization.user_sql_row.id,
    update.username,
  )
  .await
  {
    Ok(_) => match user_authorization.get_user(&state.pool).await {
      Ok(user) => axum::Json(user).into_response(),
      Err(e) => return e.into_response(),
    },
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
    (status = 200, content_type = "application/json", body = User),
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
  match update_user_password(
    &state.pool,
    &state.config,
    user_authorization.user_sql_row.id,
    update.password.as_str(),
  )
  .await
  {
    Ok(_) => match user_authorization.get_user(&state.pool).await {
      Ok(user) => axum::Json(user).into_response(),
      Err(e) => return e.into_response(),
    },
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
    (status = 200, content_type = "application/json", body = User),
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
  match crate::model::user::sql::update_user_info(
    &state.pool,
    user_authorization.user_sql_row.id,
    update.into(),
  )
  .await
  {
    Ok(_) => match user_authorization.get_user(&state.pool).await {
      Ok(user) => axum::Json(user).into_response(),
      Err(e) => return e.into_response(),
    },
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
