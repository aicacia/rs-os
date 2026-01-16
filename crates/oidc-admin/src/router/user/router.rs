use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use os_api::{
  BasicClaims, UserAuthorization,
  error::{HttpError, INTERNAL_ERROR, NOT_FOUND_ERROR},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  common::{entity::Permission, helper::has_permission},
  entity::RouterState,
  user::{
    constants::TAG,
    entity::{CreateUserRequest, UpdateUserRequest, User},
  },
};
use os_oidc_model::entities::users::{
  create_user, delete_user, get_user_by_id, list_users, update_user,
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
  user_authorization: UserAuthorization<BasicClaims>,
) -> impl IntoResponse {
  match has_permission(
    &user_authorization,
    &state.config.oidc_application_urn,
    Permission::UserRead,
  ) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  match list_users(&state.database_connection).await {
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
  user_authorization: UserAuthorization<BasicClaims>,
  Path(user_id): Path<i64>,
) -> impl IntoResponse {
  match has_permission(
    &user_authorization,
    &state.config.oidc_application_urn,
    Permission::UserRead,
  ) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_model = match get_user_by_id(&state.database_connection, user_id).await {
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

  let user: User = user_model.into();
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
  user_authorization: UserAuthorization<BasicClaims>,
  Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
  match has_permission(
    &user_authorization,
    &state.config.oidc_application_urn,
    Permission::UserWrite,
  ) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_model = match create_user(&state.database_connection, &request.username).await {
    Ok(user) => user,
    Err(e) => {
      log::error!("error creating user: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let user: User = user_model.into();
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
  user_authorization: UserAuthorization<BasicClaims>,
  Path(user_id): Path<i64>,
  Json(request): Json<UpdateUserRequest>,
) -> impl IntoResponse {
  match has_permission(
    &user_authorization,
    &state.config.oidc_application_urn,
    Permission::UserWrite,
  ) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_model = match update_user(&state.database_connection, user_id, &request.username).await {
    Ok(user) => user,
    Err(e) => {
      log::error!("error updating user: {}", e);
      return HttpError::not_found().into_response();
    }
  };

  let user: User = user_model.into();
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
  user_authorization: UserAuthorization<BasicClaims>,
  Path(user_id): Path<i64>,
) -> impl IntoResponse {
  match has_permission(
    &user_authorization,
    &state.config.oidc_application_urn,
    Permission::UserDelete,
  ) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  match delete_user(&state.database_connection, user_id).await {
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
