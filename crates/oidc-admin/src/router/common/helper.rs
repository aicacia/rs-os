use os_api::{BasicClaims, HttpError, UserAuthorization};

use crate::router::common::entity::Permission;

pub fn has_permission(
  user_authorization: &UserAuthorization<BasicClaims>,
  application_urn: &str,
  permission: Permission,
) -> Result<(), HttpError> {
  match user_authorization.has_permission(application_urn, Permission::AdminAll.as_str()) {
    Ok(_) => return Ok(()),
    Err(_e) => {}
  }
  match user_authorization.has_permission(application_urn, permission.as_str()) {
    Ok(_) => Ok(()),
    Err(e) => Err(e.into()),
  }
}
