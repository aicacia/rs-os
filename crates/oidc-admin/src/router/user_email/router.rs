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
  common::entity::Permission,
  entity::RouterState,
  user_email::{
    constants::TAG,
    entity::{CreateUserEmailRequest, UpdateUserEmailRequest, UserEmail},
  },
};
use os_oidc_model::entities::user_emails::{
  create_user_email, delete_user_email, get_user_email_by_id, list_user_emails_by_user_id,
  update_user_email_primary, verify_user_email,
};

#[utoipa::path(
  get,
  path = "/users/{user_id}/emails",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = [UserEmail]),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn list_user_emails(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization<BasicClaims>,
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

  // Users can read their own emails, admins can read any
  if user_authorization.user_info.claims.user != user_id_parsed {
    if let Err(e) = user_authorization.has_permission(&state.config.oidc_application_urn, Permission::UserRead.as_str()) {
      return e.into_response();
    }
  }

  match list_user_emails_by_user_id(&state.database_connection, user_id_parsed).await {
    Ok(emails) => {
      let emails: Vec<UserEmail> = emails.into_iter().map(|e| e.into()).collect();
      axum::Json(emails).into_response()
    }
    Err(e) => {
      log::error!("error listing user emails: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  get,
  path = "/users/{user_id}/emails/{email_id}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = UserEmail),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:read"])
  )
)]
pub async fn get_user_email(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization<BasicClaims>,
  Path((user_id, email_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let email_id_parsed = match email_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("email_id", "invalid_id")
        .into_response();
    }
  };

  // Users can read their own emails, admins can read any
  if user_authorization.user_info.claims.user != user_id_parsed {
    if let Err(e) = user_authorization.has_permission(&state.config.oidc_application_urn, Permission::UserRead.as_str()) {
      return e.into_response();
    }
  }

  let email_model = match get_user_email_by_id(&state.database_connection, email_id_parsed).await {
    Ok(Some(email)) => {
      if email.user_id != user_id_parsed {
        return HttpError::not_found()
          .with_error("email", NOT_FOUND_ERROR)
          .into_response();
      }
      email
    }
    Ok(None) => {
      return HttpError::not_found()
        .with_error("email", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("error fetching user email: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let email: UserEmail = email_model.into();
  axum::Json(email).into_response()
}

#[utoipa::path(
  post,
  path = "/users/{user_id}/emails",
  tags = [TAG],
  request_body = CreateUserEmailRequest,
  responses(
    (status = 201, content_type = "application/json", body = UserEmail),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:create"])
  )
)]
pub async fn create_user_email_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization<BasicClaims>,
  Path(user_id): Path<String>,
  Json(request): Json<CreateUserEmailRequest>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  // Users can add their own emails, admins can add any
  if user_authorization.user_info.claims.user != user_id_parsed {
    if let Err(e) = user_authorization.has_permission(&state.config.oidc_application_urn, Permission::UserCreate.as_str()) {
      return e.into_response();
    }
  }

  let email_model =
    match create_user_email(&state.database_connection, user_id_parsed, &request.email).await {
      Ok(email) => email,
      Err(e) => {
        log::error!("error creating user email: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let email: UserEmail = email_model.into();
  (axum::http::StatusCode::CREATED, axum::Json(email)).into_response()
}

#[utoipa::path(
  patch,
  path = "/users/{user_id}/emails/{email_id}",
  tags = [TAG],
  request_body = UpdateUserEmailRequest,
  responses(
    (status = 200, content_type = "application/json", body = UserEmail),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:update"])
  )
)]
pub async fn update_user_email_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization<BasicClaims>,
  Path((user_id, email_id)): Path<(String, String)>,
  Json(request): Json<UpdateUserEmailRequest>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let email_id_parsed = match email_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("email_id", "invalid_id")
        .into_response();
    }
  };

  // Users can update their own emails, admins can update any
  if user_authorization.user_info.claims.user != user_id_parsed {
    if let Err(e) = user_authorization.has_permission(&state.config.oidc_application_urn, Permission::UserUpdate.as_str()) {
      return e.into_response();
    }
  }

  if let Some(is_primary) = request.is_primary
    && is_primary
  {
    let email_model =
      match update_user_email_primary(&state.database_connection, user_id_parsed, email_id_parsed)
        .await
      {
        Ok(Some(email)) => email,
        Ok(None) => {
          return HttpError::not_found()
            .with_error("email", NOT_FOUND_ERROR)
            .into_response();
        }
        Err(e) => {
          log::error!("error updating user email primary: {}", e);
          return HttpError::internal_error()
            .with_application_error(INTERNAL_ERROR)
            .into_response();
        }
      };

    let email: UserEmail = email_model.into();
    return axum::Json(email).into_response();
  }

  // If no changes, just return the current email
  match get_user_email_by_id(&state.database_connection, email_id_parsed).await {
    Ok(Some(email)) => {
      if email.user_id != user_id_parsed {
        return HttpError::not_found()
          .with_error("email", NOT_FOUND_ERROR)
          .into_response();
      }
      let email: UserEmail = email.into();
      axum::Json(email).into_response()
    }
    Ok(None) => HttpError::not_found()
      .with_error("email", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error fetching user email: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  delete,
  path = "/users/{user_id}/emails/{email_id}",
  tags = [TAG],
  responses(
    (status = 204, description = "Email deleted successfully"),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:delete"])
  )
)]
pub async fn delete_user_email_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization<BasicClaims>,
  Path((user_id, email_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let email_id_parsed = match email_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("email_id", "invalid_id")
        .into_response();
    }
  };

  // Users can delete their own emails, admins can delete any
  if user_authorization.user_info.claims.user != user_id_parsed {
    if let Err(e) = user_authorization.has_permission(&state.config.oidc_application_urn, Permission::UserDelete.as_str()) {
      return e.into_response();
    }
  }

  match delete_user_email(&state.database_connection, user_id_parsed, email_id_parsed).await {
    Ok(Some(_)) => axum::http::StatusCode::NO_CONTENT.into_response(),
    Ok(None) => HttpError::not_found()
      .with_error("email", NOT_FOUND_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error deleting user email: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/users/{user_id}/emails/{email_id}/verify",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = UserEmail),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["user:update"])
  )
)]
pub async fn verify_user_email_handler(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization<BasicClaims>,
  Path((user_id, email_id)): Path<(String, String)>,
) -> impl IntoResponse {
  let user_id_parsed = match user_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("user_id", "invalid_id")
        .into_response();
    }
  };

  let email_id_parsed = match email_id.parse::<i64>() {
    Ok(id) => id,
    Err(_) => {
      return HttpError::bad_request()
        .with_error("email_id", "invalid_id")
        .into_response();
    }
  };

  // Only admins can manually verify emails
  if let Err(e) = user_authorization.has_permission(&state.config.oidc_application_urn, Permission::UserUpdate.as_str()) {
    return e.into_response();
  }

  let email_model =
    match verify_user_email(&state.database_connection, user_id_parsed, email_id_parsed).await {
      Ok(Some(email)) => email,
      Ok(None) => {
        return HttpError::not_found()
          .with_error("email", NOT_FOUND_ERROR)
          .into_response();
      }
      Err(e) => {
        log::error!("error verifying user email: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let email: UserEmail = email_model.into();
  axum::Json(email).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(list_user_emails))
    .routes(routes!(get_user_email))
    .routes(routes!(create_user_email_handler))
    .routes(routes!(update_user_email_handler))
    .routes(routes!(delete_user_email_handler))
    .routes(routes!(verify_user_email_handler))
    .with_state(state)
}
