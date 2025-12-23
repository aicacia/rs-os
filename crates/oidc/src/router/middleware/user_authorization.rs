use std::{
  collections::{HashMap, HashSet},
  str::FromStr,
};

use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use os_api::Claims;

use crate::router::{
  common::{
    constants::{
      AUTHORIZATION_HEADER, SCOPE_ADDRESS, SCOPE_EMAIL, SCOPE_PHONE, SCOPE_PROFILE,
      TOKEN_TYPE_BEARER,
    },
    entity::{BasicClaims, Permission, UserInfo},
  },
  entity::RouterState,
  error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, REQUIRED_ERROR},
  middleware::authorization::Authorization,
};
use os_model::entities::{
  permissions,
  roles::{self, list_roles_by_user_id},
  user_emails::get_user_primary_email_by_user_id,
  user_infos::get_user_info_by_user_id,
  user_phone_numbers::get_user_primary_phone_number_by_user_id,
  users::{self, get_user_by_id, get_user_role_permissions_by_user_id},
};

pub struct UserAuthorization {
  pub claims: BasicClaims,
  pub user_model: users::Model,
  pub role_permissions: HashMap<roles::Model, Vec<permissions::Model>>,
  pub permissions: HashSet<Permission>,
}

impl UserAuthorization {
  pub async fn get_user_info(
    &self,
    db: &sea_orm::DatabaseConnection,
  ) -> Result<UserInfo, HttpError> {
    let user_id = self.user_model.id;

    let mut user_info: UserInfo = self.user_model.clone().into();

    user_info.basic_claims = self.claims.clone();

    let has_profile = self.claims.has_scope(SCOPE_PROFILE);
    let has_email = self.claims.has_scope(SCOPE_EMAIL);
    let has_phone_number = self.claims.has_scope(SCOPE_PHONE);
    let has_address = self.claims.has_scope(SCOPE_ADDRESS);

    if has_profile {
      user_info.permissions = self.permissions.iter().cloned().collect();
      user_info.roles = match list_roles_by_user_id(db, user_id).await {
        Ok(roles) => roles.into_iter().map(|r| r.uri).collect(),
        Err(e) => {
          log::error!("error fetching user roles: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      };
    }

    if has_profile || has_address {
      match get_user_info_by_user_id(db, user_id).await {
        Ok(Some(info)) => {
          if has_profile {
            user_info.profile = info.into();
          }
          if !has_address {
            user_info.profile.address = None;
          }
        }
        Ok(None) => {}
        Err(e) => {
          log::error!("error fetching user info: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      }
    }

    if has_email {
      match get_user_primary_email_by_user_id(db, user_id).await {
        Ok(Some(email)) => {
          user_info.profile.email = Some(email.email.to_owned());
          user_info.profile.email_verified = Some(email.is_verified());
        }
        Ok(None) => {}
        Err(e) => {
          log::error!("error fetching user emails: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      }
    }
    if has_phone_number {
      match get_user_primary_phone_number_by_user_id(db, user_id).await {
        Ok(Some(phone_number)) => {
          user_info.profile.phone = Some(phone_number.phone_number.to_owned());
          user_info.profile.phone_verified = Some(phone_number.is_verified());
        }
        Ok(None) => {}
        Err(e) => {
          log::error!("error fetching user phone numbers: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      }
    }

    Ok(user_info)
  }

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

    match get_user_by_id(&router_state.database_connection, user_id).await {
      Ok(Some(user_model)) => {
        if !user_model.is_active() {
          log::error!("invalid authorization user is not active");
          return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
        }
        let role_permissions = match get_user_role_permissions_by_user_id(
          &router_state.database_connection,
          user_model.id,
        )
        .await
        {
          Ok(role_permissions) => role_permissions,
          Err(e) => {
            log::error!("failed to fetch permissions: {}", e);
            return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
          }
        };

        let mut permissions: HashSet<Permission> = HashSet::default();
        for (_role_id, perms) in role_permissions.iter() {
          for p in perms {
            if let Ok(permission) = Permission::from_str(&p.uri) {
              permissions.insert(permission);
            }
          }
        }

        return Ok(Self {
          claims: authorization.claims,
          user_model,
          role_permissions,
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
