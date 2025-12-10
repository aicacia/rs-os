use os_model::entities::{prelude::*, *};
use sea_orm::*;

// Type aliases for backward compatibility
pub type RoleModel = roles::Model;
pub type PermissionModel = permissions::Model;
pub type RolePermissionModel = roles_permissions::Model;

pub async fn get_role_permissions_by_role_id(
  db: &DatabaseConnection,
  role_id: i64,
) -> Result<Vec<PermissionModel>, DbErr> {
  // Query permissions through the roles_permissions join table
  let role_permissions = RolesPermissions::find()
    .filter(roles_permissions::Column::RoleId.eq(role_id))
    .all(db)
    .await?;

  let permission_ids: Vec<i64> = role_permissions.iter().map(|rp| rp.permission_id).collect();

  if permission_ids.is_empty() {
    return Ok(Vec::new());
  }

  Permissions::find()
    .filter(permissions::Column::Id.is_in(permission_ids))
    .all(db)
    .await
}
