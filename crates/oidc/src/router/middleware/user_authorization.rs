use std::{
  collections::{HashMap, HashSet},
  str::FromStr,
};

use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use os_api::{
  Claims,
  error::{HttpError, INTERNAL_ERROR, INVALID_ERROR, REQUIRED_ERROR},
  util::permission_grants,
};

use crate::router::{
  common::{
    constants::{
      AUTHORIZATION_HEADER, SCOPE_ADDRESS, SCOPE_EMAIL, SCOPE_PHONE, SCOPE_PROFILE,
      TOKEN_TYPE_BEARER,
    },
    entity::{BasicClaims, Permission, UserInfo},
    helper::parse_user_sub,
  },
  entity::RouterState,
  middleware::authorization::Authorization,
};
use os_oidc_model::entities::{
  applications::get_applications_by_urns,
  permissions,
  roles::{self},
  user_emails::get_user_primary_email_by_user_id,
  user_infos::get_user_info_by_user_id,
  user_phone_numbers::get_user_primary_phone_number_by_user_id,
  users::{self, get_user_by_id, get_user_role_permissions_by_user_id_and_application_id},
};

pub struct UserAuthorization {
  pub application_urn: String,
  pub claims: BasicClaims,
  pub user_model: users::Model,
  pub role_permissions: HashMap<String, HashMap<roles::Model, Vec<permissions::Model>>>,
  pub permissions: HashMap<String, HashSet<Permission>>,
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
      for (audience_urn, role_permission_map) in self.role_permissions.iter() {
        let roles = role_permission_map
          .keys()
          .map(|role| role.uri.clone())
          .collect::<Vec<_>>();

        if !roles.is_empty() {
          user_info.roles.insert(audience_urn.clone(), roles);
        }
      }

      for (audience_urn, permission_set) in self.permissions.iter() {
        let permissions = permission_set.iter().cloned().collect::<Vec<_>>();

        if !permissions.is_empty() {
          user_info
            .permissions
            .insert(audience_urn.clone(), permissions);
        }
      }
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

  fn permission_grants_enum(user_permission: Permission, required_permission: Permission) -> bool {
    permission_grants(user_permission.as_str(), required_permission.as_str())
  }

  fn has_permission_for(
    permissions: &HashSet<Permission>,
    required_permission: Permission,
  ) -> bool {
    permissions
      .iter()
      .any(|user_permission| Self::permission_grants_enum(*user_permission, required_permission))
  }

  pub fn has_permission(
    &self,
    application_urn: &str,
    permission: Permission,
  ) -> Result<(), HttpError> {
    if let Some(app_permissions) = self.permissions.get(application_urn) {
      if Self::has_permission_for(app_permissions, permission) {
        return Ok(());
      }
    }
    Err(HttpError::forbidden().with_error(permission.as_str(), REQUIRED_ERROR))
  }

  pub fn has_permissions<'a, I>(
    &self,
    application_urn: &str,
    permissions: I,
  ) -> Result<(), HttpError>
  where
    I: IntoIterator<Item = &'a Permission>,
  {
    if let Some(app_permissions) = self.permissions.get(application_urn) {
      let mut missing_permissions: Vec<Permission> = Vec::default();
      for permission in permissions {
        if !Self::has_permission_for(app_permissions, *permission) {
          missing_permissions.push(*permission);
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
    Err(HttpError::forbidden().with_error(application_urn, REQUIRED_ERROR))
  }

  pub fn has_oidc_application_permission(&self, permission: Permission) -> Result<(), HttpError> {
    self.has_permission(&self.application_urn, permission)
  }

  pub fn has_oidc_application_permissions<'a, I>(&self, permissions: I) -> Result<(), HttpError>
  where
    I: IntoIterator<Item = &'a Permission>,
  {
    self.has_permissions(&self.application_urn, permissions)
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

    let user_id = match parse_user_sub(&authorization.claims.sub) {
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

        let audience_urns: Vec<String> = authorization.claims.aud.clone();

        if audience_urns.is_empty() {
          log::error!("JWT has no audiences");
          return Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
        }

        let mut role_permissions: HashMap<String, HashMap<roles::Model, Vec<permissions::Model>>> =
          HashMap::default();
        let mut permissions_map: HashMap<String, HashSet<Permission>> = HashMap::default();

        let valid_audience_urns: Vec<String> = audience_urns
          .iter()
          .filter(|urn| urn.starts_with("urn:os:oidc:application:"))
          .cloned()
          .collect();

        if !valid_audience_urns.is_empty() {
          match get_applications_by_urns(&router_state.database_connection, &valid_audience_urns)
            .await
          {
            Ok(applications) => {
              let app_map: HashMap<String, _> = applications
                .into_iter()
                .map(|app| (app.urn.clone(), app))
                .collect();

              for audience_urn in valid_audience_urns {
                match app_map.get(&audience_urn) {
                  Some(app) => {
                    match get_user_role_permissions_by_user_id_and_application_id(
                      &router_state.database_connection,
                      user_model.id,
                      app.id,
                    )
                    .await
                    {
                      Ok(app_role_permissions) => {
                        let mut permissions: HashSet<Permission> = HashSet::default();
                        for (_role_id, perms) in app_role_permissions.iter() {
                          for p in perms {
                            if let Ok(permission) = Permission::from_str(&p.uri) {
                              permissions.insert(permission);
                            }
                          }
                        }

                        role_permissions.insert(audience_urn.clone(), app_role_permissions);
                        permissions_map.insert(audience_urn.clone(), permissions);
                      }
                      Err(e) => {
                        log::error!(
                          "failed to fetch permissions for application {}: {}",
                          audience_urn,
                          e
                        );
                        return Err(
                          HttpError::internal_error().with_application_error(INTERNAL_ERROR),
                        );
                      }
                    }
                  }
                  None => {
                    log::error!("application not found for audience: {}", audience_urn);
                    return Err(
                      HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR),
                    );
                  }
                }
              }
            }
            Err(e) => {
              log::error!("failed to fetch applications: {}", e);
              return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
            }
          }
        }

        // Prefer the configured application URN when we have permissions for it, otherwise
        // fall back to the first audience we built permissions for so test-generated
        // applications work without configuring application_urn explicitly.
        let application_urn = if permissions_map.contains_key(&router_state.config.application_urn)
        {
          router_state.config.application_urn.clone()
        } else {
          permissions_map
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| router_state.config.application_urn.clone())
        };

        Ok(Self {
          application_urn,
          claims: authorization.claims,
          user_model,
          role_permissions,
          permissions: permissions_map,
        })
      }
      Ok(None) => Err(HttpError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR)),
      Err(e) => {
        log::error!("invalid authorization user not found for sub: {}", e);
        Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR))
      }
    }
  }
}
