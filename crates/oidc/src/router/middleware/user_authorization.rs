use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

use crate::{
  model::user::sql::{UserSQLRow, get_user_by_id},
  router::{
    common::{
      constants::{AUTHORIZATION_HEADER, TOKEN_TYPE_BEARER},
      entity::BasicClaims,
    },
    entity::RouterState,
    error::{HttpError, INVALID_ERROR},
    middleware::authorization::Authorization,
  },
};

pub struct UserAuthorization {
  pub claims: BasicClaims,
  pub user_sql_row: UserSQLRow,
}

impl<S> FromRequestParts<S> for UserAuthorization
where
  RouterState: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = HttpError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let router_state = RouterState::from_ref(state);
    let authorization = Authorization::<BasicClaims>::from_request_parts(parts, state).await?;

    if authorization.claims.r#type != TOKEN_TYPE_BEARER {
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, "invalid-token-type"));
    }

    match get_user_by_id(&router_state.pool, authorization.claims.sub).await {
      Ok(Some(user_sql_row)) => {
        if !user_sql_row.is_active() {
          log::error!("invalid authorization user is not active");
          return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
        }
        return Ok(Self {
          claims: authorization.claims,
          user_sql_row,
        });
      }
      Ok(None) => {
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      Err(e) => {
        log::error!("invalid authorization user not found for sub: {}", e);
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
    }
  }
}
