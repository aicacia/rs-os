use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  common::entity::Permission,
  entity::RouterState,
  error::{HttpError, INTERNAL_ERROR, NOT_FOUND_ERROR},
  middleware::user_authorization::UserAuthorization,
  user_phone_number::{
    constants::TAG,
    entity::{CreateUserPhoneNumberRequest, UpdateUserPhoneNumberRequest, UserPhoneNumber},
  },
};
use os_model::entities::user_phone_numbers::{
  create_user_phone_number, delete_user_phone_number, get_user_phone_number_by_id,
  list_user_phone_numbers_by_user_id, update_user_phone_number_primary, verify_user_phone_number,
};

#[utoipa::path(
  get,
  path = "/users/{user_id}/phone-numbers",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = [UserPhoneNumber]),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn list_user_phone_numbers(
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

  // Users can read their own phone numbers, admins can read any
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match list_user_phone_numbers_by_user_id(&state.database_connection, user_id_parsed).await {
    Ok(phone_numbers) => {
      let phone_numbers: Vec<UserPhoneNumber> =
        phone_numbers.into_iter().map(|p| p.into()).collect();
      axum::Json(phone_numbers).into_response()
    }
    Err(e) => {
      log::error!("error listing user phone numbers: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  get,
  path = "/users/{user_id}/phone-numbers/{phone_id}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = UserPhoneNumber),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn get_user_phone_number(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path((user_id, phone_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let phone_id_parsed = match phone_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("phone_id", "invalid_id")
        .into_response();
    }
  };

  // Users can read their own phone numbers, admins can read any
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserRead) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  let phone_model =
    match get_user_phone_number_by_id(&state.database_connection, phone_id_parsed).await {
      Ok(Some(phone)) => {
        if phone.user_id != user_id_parsed {
          return HttpError::not_found()
            .with_error("phone_number", NOT_FOUND_ERROR)
            .into_response();
        }
        phone
      }
      Ok(None) => {
        return HttpError::not_found()
          .with_error("phone_number", NOT_FOUND_ERROR)
          .into_response();
      }
      Err(e) => {
        log::error!("error fetching user phone number: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let phone: UserPhoneNumber = phone_model.into();
  axum::Json(phone).into_response()
}

#[utoipa::path(
  post,
  path = "/users/{user_id}/phone-numbers",
  tags = [TAG],
  request_body = CreateUserPhoneNumberRequest,
  responses(
    (status = 201, content_type = "application/json", body = UserPhoneNumber),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn create_user_phone_number_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(user_id): Path<String>,
  Json(request): Json<CreateUserPhoneNumberRequest>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  // Users can add their own phone numbers, admins can add any
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserWrite) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  let phone_model = match create_user_phone_number(
    &state.database_connection,
    user_id_parsed,
    &request.phone_number,
  )
  .await
  {
    Ok(phone) => phone,
    Err(e) => {
      log::error!("error creating user phone number: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let phone: UserPhoneNumber = phone_model.into();
  (axum::http::StatusCode::CREATED, axum::Json(phone)).into_response()
}

#[utoipa::path(
  patch,
  path = "/users/{user_id}/phone-numbers/{phone_id}",
  tags = [TAG],
  request_body = UpdateUserPhoneNumberRequest,
  responses(
    (status = 200, content_type = "application/json", body = UserPhoneNumber),
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
pub async fn update_user_phone_number_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path((user_id, phone_id)): Path<(String, String)>,
  Json(request): Json<UpdateUserPhoneNumberRequest>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let phone_id_parsed = match phone_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("phone_id", "invalid_id")
        .into_response();
    }
  };

  // Users can update their own phone numbers, admins can update any
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserWrite) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  if let Some(is_primary) = request.is_primary {
    if is_primary {
      let phone_model = match update_user_phone_number_primary(
        &state.database_connection,
        user_id_parsed,
        phone_id_parsed,
      )
      .await
      {
        Ok(Some(phone)) => phone,
        Ok(None) => {
          return HttpError::not_found()
            .with_error("phone_number", NOT_FOUND_ERROR)
            .into_response();
        }
        Err(e) => {
          log::error!("error updating user phone number primary: {}", e);
          return HttpError::internal_error()
            .with_application_error(INTERNAL_ERROR)
            .into_response();
        }
      };

      let phone: UserPhoneNumber = phone_model.into();
      return axum::Json(phone).into_response();
    }
  }

  // If no changes, just return the current phone number
  match get_user_phone_number_by_id(&state.database_connection, phone_id_parsed).await {
    Ok(Some(phone)) => {
      if phone.user_id != user_id_parsed {
        return HttpError::not_found()
          .with_error("phone_number", NOT_FOUND_ERROR)
          .into_response();
      }
      let phone: UserPhoneNumber = phone.into();
      axum::Json(phone).into_response()
    }
    Ok(None) => HttpError::not_found()
      .with_error("phone_number", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error fetching user phone number: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  delete,
  path = "/users/{user_id}/phone-numbers/{phone_id}",
  tags = [TAG],
  responses(
    (status = 204, description = "Phone number deleted successfully"),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn delete_user_phone_number_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path((user_id, phone_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let phone_id_parsed = match phone_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("phone_id", "invalid_id")
        .into_response();
    }
  };

  // Users can delete their own phone numbers, admins can delete any
  if user_authorization.user_model.id != user_id_parsed {
    match user_authorization.has_permission(Permission::UserWrite) {
      Ok(_) => {}
      Err(e) => return e.into_response(),
    }
  }

  match delete_user_phone_number(&state.database_connection, user_id_parsed, phone_id_parsed).await
  {
    Ok(Some(_)) => axum::http::StatusCode::NO_CONTENT.into_response(),
    Ok(None) => HttpError::not_found()
      .with_error("phone_number", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error deleting user phone number: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/users/{user_id}/phone-numbers/{phone_id}/verify",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = UserPhoneNumber),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:write"])
  )
)]
pub async fn verify_user_phone_number_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path((user_id, phone_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let phone_id_parsed = match phone_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("phone_id", "invalid_id")
        .into_response();
    }
  };

  // Only admins can manually verify phone numbers
  match user_authorization.has_permission(Permission::UserWrite) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  }

  let phone_model =
    match verify_user_phone_number(&state.database_connection, user_id_parsed, phone_id_parsed)
      .await
    {
      Ok(Some(phone)) => phone,
      Ok(None) => {
        return HttpError::not_found()
          .with_error("phone_number", NOT_FOUND_ERROR)
          .into_response();
      }
      Err(e) => {
        log::error!("error verifying user phone number: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let phone: UserPhoneNumber = phone_model.into();
  axum::Json(phone).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(list_user_phone_numbers))
    .routes(routes!(get_user_phone_number))
    .routes(routes!(create_user_phone_number_handler))
    .routes(routes!(update_user_phone_number_handler))
    .routes(routes!(delete_user_phone_number_handler))
    .routes(routes!(verify_user_phone_number_handler))
    .with_state(state)
}
