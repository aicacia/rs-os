use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use os_model::entities::users::{
  assign_user_role, get_user_roles_by_user_id, remove_user_role,
};
use crate::{
  core::user_permissions::get_user_permissions,
  router::{
    common::permissions::Permission,
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, NOT_FOUND_ERROR},
    middleware::user_authorization::UserAuthorization,
    user_role::{
      constants::TAG,
      entity::{AssignUserRoleRequest, UserPermissions, UserRole},
    },
  },
};

#[utoipa::path(
  get,
  path = "/users/{user_id}/roles",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = [UserRole]),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn list_user_roles(
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

  // Users can read their own roles, admins can read any
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match get_user_roles_by_user_id(&state.database, user_id_parsed).await {
    Ok(role_tuples) => {
      let roles: Vec<UserRole> = role_tuples
        .into_iter()
        .filter_map(|(_, role_opt)| role_opt.map(|r| r.into()))
        .collect();
      axum::Json(roles).into_response()
    }
    Err(e) => {
      log::error!("error listing user roles: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  get,
  path = "/users/{user_id}/permissions",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = UserPermissions),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn list_user_permissions(
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

  // Users can read their own permissions, admins can read any
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match get_user_permissions(&state.database, user_id_parsed).await {
    Ok(permissions) => axum::Json(UserPermissions { permissions }).into_response(),
    Err(e) => {
      log::error!("error listing user permissions: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/users/{user_id}/roles",
  tags = [TAG],
  request_body = AssignUserRoleRequest,
  responses(
    (status = 201, content_type = "application/json", body = UserRole),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn assign_user_role_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(user_id): Path<String>,
  Json(request): Json<AssignUserRoleRequest>,
) -> impl IntoResponse {
  // Only admins can assign roles
  match user_authorization.has_permission(Permission::UserWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let role_model = match assign_user_role(&state.database, user_id_parsed, request.role_id).await {
    Ok(role) => role,
    Err(e) => {
      log::error!("error assigning user role: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let role: UserRole = role_model.into();
  (axum::http::StatusCode::CREATED, axum::Json(role)).into_response()
}

#[utoipa::path(
  delete,
  path = "/users/{user_id}/roles/{role_id}",
  tags = [TAG],
  responses(
    (status = 204, description = "Role removed successfully"),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn remove_user_role_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path((user_id, role_id)): Path<(String, String)>,
) -> impl IntoResponse {
  // Only admins can remove roles
  match user_authorization.has_permission(Permission::UserWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let role_id_parsed = match role_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("role_id", "invalid_id")
        .into_response();
    }
  };

  match remove_user_role(&state.database, user_id_parsed, role_id_parsed).await {
    Ok(Some(_)) => axum::http::StatusCode::NO_CONTENT.into_response(),
    Ok(None) => HttpError::not_found()
      .with_error("role", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error removing user role: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(list_user_roles))
    .routes(routes!(list_user_permissions))
    .routes(routes!(assign_user_role_handler))
    .routes(routes!(remove_user_role_handler))
    .with_state(state)
}
