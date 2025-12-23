use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use os_api::Authorization;
use os_api::error::INVALID_TYPE_ERROR;

use crate::router::{
  common::{
    constants::{
      AUTHORIZATION_HEADER, SCOPE_ADDRESS, SCOPE_EMAIL, SCOPE_PHONE, SCOPE_PROFILE,
      TOKEN_TYPE_BEARER,
    },
    entity::{BasicClaims, Claims},
    permissions::Permission,
  },
  current_user::entity::{CurrentUser, UserOAuth2Provider, UserRole},
  entity::RouterState,
  error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, REQUIRED_ERROR},
};
use os_model::entities::{
  permissions, roles,
  user_emails::list_user_emails_by_user_id,
  user_infos::get_user_info_by_user_id,
  user_o_auth2_providers::get_user_oauth2_providers,
  user_phone_numbers::list_user_phone_numbers_by_user_id,
  users::{self, get_user_by_id, get_user_role_permissions_by_user_id, get_user_roles_by_user_id},
};

pub struct UserAuthorization {
  pub claims: BasicClaims,
  pub user_model: users::Model,
  pub role_permissions: HashMap<roles::Model, Vec<permissions::Model>>,
  pub permissions: HashSet<Permission>,
}

impl UserAuthorization {
  pub async fn get_user(&self, db: &sea_orm::DatabaseConnection) -> Result<CurrentUser, HttpError> {
    let mut user: CurrentUser = self.user_model.clone().into();

    let has_profile = self.claims.has_scope(SCOPE_PROFILE);
    let has_email = self.claims.has_scope(SCOPE_EMAIL);
    let has_phone_number = self.claims.has_scope(SCOPE_PHONE);
    let has_address = self.claims.has_scope(SCOPE_ADDRESS);

    if has_profile {
      let role_tuples = match get_user_roles_by_user_id(db, user.id).await {
        Ok(roles) => roles,
        Err(e) => {
          log::error!("error fetching user roles: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      };

      for (_, role_opt) in role_tuples {
        if let Some(role_model) = role_opt {
          let permissions = if let Some(permissions) = self.role_permissions.get(&role_model) {
            permissions
              .into_iter()
              .map(|p| p.uri.clone())
              .collect::<Vec<_>>()
          } else {
            Vec::default()
          };
          let mut role: UserRole = role_model.into();
          role.permissions = permissions;
          user.roles.push(role);
        }
      }

      user.info = match get_user_info_by_user_id(db, user.id).await {
        Ok(Some(user_info_model)) => {
          let mut user_info: crate::router::current_user::entity::UserInfo = user_info_model.into();
          if !has_address {
            user_info.address = None;
          }
          Some(user_info)
        }
        Ok(None) => None,
        Err(e) => {
          log::error!("error fetching user info: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      };
      user.oauth2_providers = match get_user_oauth2_providers(db, user.id).await {
        Ok(oauth2_providers) => oauth2_providers
          .into_iter()
          .filter_map(|(provider, provider_info_opt)| {
            provider_info_opt.map(|provider_info| {
              let mut oauth2_provider = UserOAuth2Provider {
                id: provider_info.id,
                uri: provider_info.uri,
                name: provider_info.description,
                email: Some(provider.email),
                updated_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.updated_at, 0)
                  .unwrap_or_default(),
                created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(provider.created_at, 0)
                  .unwrap_or_default(),
              };

              if !has_email && !has_profile {
                oauth2_provider.email = None;
              }

              oauth2_provider
            })
          })
          .collect(),
        Err(e) => {
          log::error!("error fetching user oauth2 providers: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      };
    }
    if has_email {
      match list_user_emails_by_user_id(db, user.id).await {
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
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      }
    }
    if has_phone_number {
      match list_user_phone_numbers_by_user_id(db, user.id).await {
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
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      }
    }
    Ok(user)
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
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_TYPE_ERROR));
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
