use sea_orm::DatabaseConnection;

use crate::router::common::permissions::Permission;

/// Get user permissions as admin Permission enum values
pub async fn get_user_permissions(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<Permission>, sea_orm::DbErr> {
  let role_permissions =
    os_model::entities::users::get_user_role_permissions_by_user_id(db, user_id).await?;

  let mut permissions = Vec::new();
  for (_role_id, perms) in role_permissions {
    for perm in perms {
      if let Ok(permission) = perm.uri.parse::<Permission>() {
        if !permissions.contains(&permission) {
          permissions.push(permission);
        }
      }
    }
  }

  Ok(permissions)
}
