use std::str::FromStr;

use axum::extract::{FromRef, FromRequestParts};
use hashbrown::{HashMap, HashSet};
use http::request::Parts;

use crate::{
  model::{
    rbac::sql::PermissionSQLRow,
    user::sql::{
      UserSQLRow, get_user_by_id, get_user_emails_by_user_id, get_user_info_by_user_id,
      get_user_oauth2_providers, get_user_phone_numbers_by_user_id,
      get_user_role_permissions_by_user_id, get_user_roles_by_user_id,
    },
  },
  router::{
    common::{
      constants::{
        AUTHORIZATION_HEADER, SCOPE_ADDRESS, SCOPE_EMAIL, SCOPE_PHONE, SCOPE_PROFILE,
        TOKEN_TYPE_BEARER,
      },
      entity::{BasicClaims, Claims},
      permissions::Permission,
    },
    current_user::entity::{User, UserOAuth2Provider, UserRole},
    entity::RouterState,
    error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, REQUIRED_ERROR},
    middleware::authorization::Authorization,
  },
};

pub struct UserAuthorization {
  pub claims: BasicClaims,
  pub user_sql_row: UserSQLRow,
  pub permission_sql_rows: HashMap<i64, Vec<PermissionSQLRow>>,
  pub permissions: HashSet<Permission>,
}

impl UserAuthorization {
  pub async fn get_user(&self, pool: &sqlx::AnyPool) -> Result<User, HttpError> {
    let mut user: User = self.user_sql_row.clone().into();

    let has_profile = self.claims.has_scope(SCOPE_PROFILE);
    let has_email = self.claims.has_scope(SCOPE_EMAIL);
    let has_phone_number = self.claims.has_scope(SCOPE_PHONE);
    let has_address = self.claims.has_scope(SCOPE_ADDRESS);

    if has_profile {
      let role_sql_rows = match get_user_roles_by_user_id(pool, user.id).await {
        Ok(roles) => roles,
        Err(e) => {
          log::error!("error fetching user roles: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      };

      for role_sql_row in role_sql_rows {
        let permissions = if let Some(permissions) = self.permission_sql_rows.get(&role_sql_row.id)
        {
          permissions
            .into_iter()
            .map(|p| p.uri.clone())
            .collect::<Vec<_>>()
        } else {
          Vec::default()
        };
        let mut role: UserRole = role_sql_row.into();
        role.permissions = permissions;
        user.roles.push(role);
      }

      user.info = match get_user_info_by_user_id(pool, user.id).await {
        Ok(Some(mut user_info)) => {
          if !has_address {
            user_info.address = None;
          }
          Some(user_info.into())
        }
        Ok(None) => None,
        Err(e) => {
          log::error!("error fetching user info: {}", e);
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      };
      user.oauth2_providers = match get_user_oauth2_providers(pool, user.id).await {
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
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }
      };
    }
    if has_email {
      match get_user_emails_by_user_id(pool, user.id).await {
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
      match get_user_phone_numbers_by_user_id(pool, user.id).await {
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
      return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, "invalid-token-type"));
    }

    match get_user_by_id(&router_state.pool, authorization.claims.sub).await {
      Ok(Some(user_sql_row)) => {
        if !user_sql_row.is_active() {
          log::error!("invalid authorization user is not active");
          return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
        }
        let permission_sql_rows =
          match get_user_role_permissions_by_user_id(&router_state.pool, user_sql_row.id).await {
            Ok(permission_sql_rows) => permission_sql_rows,
            Err(e) => {
              log::error!("failed to fetch permissions: {}", e);
              return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
            }
          };

        let mut permissions: HashSet<Permission> = HashSet::default();
        for (_role_id, perms) in permission_sql_rows.iter() {
          for p in perms {
            if let Ok(permission) = Permission::from_str(&p.uri) {
              permissions.insert(permission);
            }
          }
        }

        return Ok(Self {
          claims: authorization.claims,
          user_sql_row,
          permission_sql_rows,
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
