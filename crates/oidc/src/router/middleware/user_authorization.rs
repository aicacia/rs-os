use std::str::FromStr;
use std::collections::HashMap;

use axum::extract::{FromRef, FromRequestParts};
use hashbrown::HashSet;
use http::request::Parts;
use os_model::entities::{permissions, users};

use crate::{
  router::{
    common::{
      constants::{AUTHORIZATION_HEADER, TOKEN_TYPE_BEARER},
      entity::BasicClaims,
      permissions::Permission,
    },
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, REQUIRED_ERROR},
    middleware::authorization::Authorization,
  },
};

pub struct UserAuthorization {
  pub claims: BasicClaims,
  pub user_model: users::Model,
  pub permission_models: HashMap<i64, Vec<permissions::Model>>,
  pub permissions: HashSet<Permission>,
}

impl UserAuthorization {
  pub fn has_permission(&self, permission: Permission) -> Result<(), HttpError> {
    if self.permissions.contains(&Permission::AdminAll) || self.permissions.contains(&permission) {
      return Ok(());
    }
    Err(HttpError::forbidden().with_error(permission.as_str(), REQUIRED_ERROR))
  }

  pub fn has_permissions<'a, I>(&self, permissions: I) -> Result<(), HttpError>
  where
    I: IntoIterator<Item = &'a Permission>,
  {
    if self.permissions.contains(&Permission::AdminAll) {
      return Ok(());
    }
    let mut missing_permissions: HashSet<&Permission> = HashSet::default();
    for permission in permissions {
      if !self.permissions.contains(permission) {
        missing_permissions.insert(permission);
      }
    }
    if missing_permissions.is_empty() {
      return Ok(());
    }
    let mut e = HttpError::forbidden();
    for missing_permission in missing_permissions {
      e = e.with_error(missing_permission.as_str(), REQUIRED_ERROR);
    }
    return Err(e);
  }
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

    let user_id = match authorization.claims.sub.parse::<i64>() {
      Ok(id) => id,
      Err(e) => {
        log::error!(
          "invalid authorization sub claim is not a valid user id: {}",
          e
        );
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
    };

    match users::get_user_by_id(&router_state.database, user_id).await {
      Ok(Some(user_model)) => {
        if !user_model.is_active() {
          log::error!("invalid authorization user is not active");
          return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
        }
        let permission_models =
          match users::get_user_role_permissions_by_user_id(&router_state.database, user_model.id).await {
            Ok(permission_models) => permission_models,
            Err(e) => {
              log::error!("failed to fetch permissions: {}", e);
              return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
            }
          };

        let mut permissions: HashSet<Permission> = HashSet::default();
        for (_role_id, perms) in permission_models.iter() {
          for p in perms {
            if let Ok(permission) = Permission::from_str(&p.uri) {
              permissions.insert(permission);
            }
          }
        }

        return Ok(Self {
          claims: authorization.claims,
          user_model,
          permission_models,
          permissions,
        });
      }
      Ok(None) => {
        return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
      }
      Err(e) => {
        log::error!("invalid authorization user not found for sub: {}", e);
        return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
      }
    }
  }
}
