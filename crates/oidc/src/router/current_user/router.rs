use axum::{extract::State, response::IntoResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  model::user::sql::{
    get_user_emails_by_user_id, get_user_info_by_user_id, get_user_oauth2_providers,
    get_user_phone_numbers_by_user_id,
  },
  router::{
    common::helper::{
      has_address_scope, has_email_scope, has_phone_number_scope, has_profile_scope,
    },
    current_user::{
      constants::TAG,
      entity::{User, UserOAuth2Provider},
    },
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR},
    middleware::user_authorization::UserAuthorization,
  },
};

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
  UserAuthorization {
    user_sql_row,
    claims,
  }: UserAuthorization,
) -> impl IntoResponse {
  let mut user: User = user_sql_row.into();

  let has_profile = has_profile_scope(&claims.scopes);
  let has_email = has_email_scope(&claims.scopes);
  let has_phone_number = has_phone_number_scope(&claims.scopes);
  let has_address = has_address_scope(&claims.scopes);

  if has_profile {
    user.info = match get_user_info_by_user_id(&state.pool, user.id).await {
      Ok(Some(user_info)) => user_info.into(),
      Ok(None) => Default::default(),
      Err(e) => {
        log::error!("error fetching user oauth2 providers: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };
    if !has_address {
      user.info.address = None;
    }
    user.oauth2_providers = match get_user_oauth2_providers(&state.pool, user.id).await {
      Ok(oauth2_providers) => oauth2_providers
        .into_iter()
        .map(|op| {
          let mut oauth2_provider: UserOAuth2Provider = op.into();

          if !has_email && !has_profile {
            oauth2_provider.email = None;
          }

          oauth2_provider
        })
        .collect(),
      Err(e) => {
        log::error!("error fetching user oauth2 providers: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };
  }
  if has_email {
    match get_user_emails_by_user_id(&state.pool, user.id).await {
      Ok(emails) => {
        for email in emails {
          if email.is_primary() {
            user.email = Some(email.into());
          } else {
            user.emails.push(email.into());
          }
        }
      }
      Err(e) => {
        log::error!("error fetching user emails: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    }
  }
  if has_phone_number {
    match get_user_phone_numbers_by_user_id(&state.pool, user.id).await {
      Ok(phone_numbers) => {
        for phone_number in phone_numbers {
          if phone_number.is_primary() {
            user.phone_number = Some(phone_number.into());
          } else {
            user.phone_numbers.push(phone_number.into());
          }
        }
      }
      Err(e) => {
        log::error!("error fetching user phone numbers: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    }
  }

  axum::Json(user).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(current_user))
    .with_state(state)
}
