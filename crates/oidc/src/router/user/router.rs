use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  model::user::orm::{create_user, delete_user, get_user_by_id, list_users, update_user},
  router::{
    common::permissions::Permission,
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, NOT_FOUND_ERROR},
    middleware::user_authorization::UserAuthorization,
    user::{
      constants::TAG,
      entity::{CreateUserRequest, UpdateUserRequest, User},
    },
  },
};

#[utoipa::path(
  get,
  path = "/users",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = [User]),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn user_list(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::UserRead) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  match list_users(&state.database).await {
    Ok(users) => {
      let users: Vec<User> = users.into_iter().map(|u| u.into()).collect();
      axum::Json(users).into_response()
    }
    Err(e) => {
      log::error!("error listing users: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  get,
  path = "/users/{id}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = User),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn get_user(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(user_id): Path<i64>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::UserRead) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_sql_row = match get_user_by_id(&state.database, user_id).await {
    Ok(Some(user)) => user,
    Ok(None) => {
      return HttpError::not_found()
        .with_error("user", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let user: User = user_sql_row.into();
  axum::Json(user).into_response()
}

#[utoipa::path(
  post,
  path = "/users",
  tags = [TAG],
  request_body = CreateUserRequest,
  responses(
    (status = 201, content_type = "application/json", body = User),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn create_user_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::UserWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_sql_row = match create_user(&state.database, &request.username).await {
    Ok(user) => user,
    Err(e) => {
      log::error!("error creating user: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let user: User = user_sql_row.into();
  (axum::http::StatusCode::CREATED, axum::Json(user)).into_response()
}

#[utoipa::path(
  put,
  path = "/users/{id}",
  tags = [TAG],
  request_body = UpdateUserRequest,
  responses(
    (status = 200, content_type = "application/json", body = User),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn update_user_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(user_id): Path<i64>,
  Json(request): Json<UpdateUserRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::UserWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_sql_row = match update_user(&state.database, user_id, &request.username).await {
    Ok(user) => user,
    Err(e) => {
      log::error!("error updating user: {}", e);
      return HttpError::not_found().into_response();
    }
  };

  let user: User = user_sql_row.into();
  axum::Json(user).into_response()
}

#[utoipa::path(
  delete,
  path = "/users/{id}",
  tags = [TAG],
  responses(
    (status = 204, description = "User deleted successfully"),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:delete"])
  )
)]
pub async fn delete_user_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(user_id): Path<i64>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::UserDelete) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  match delete_user(&state.database, user_id).await {
    Ok(Some(_)) => axum::http::StatusCode::NO_CONTENT.into_response(),
    Ok(None) => HttpError::not_found()
      .with_error("user", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error deleting user: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(user_list))
    .routes(routes!(get_user))
    .routes(routes!(create_user_handler))
    .routes(routes!(update_user_handler))
    .routes(routes!(delete_user_handler))
    .with_state(state)
}
