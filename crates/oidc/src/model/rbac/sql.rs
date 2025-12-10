// Re-export ORM types for backward compatibility
pub use super::orm::{
  PermissionModel as PermissionSQLRow, RoleModel as RoleSQLRow,
  RolePermissionModel as RolePermissionSQLRow, get_role_permissions_by_role_id,
};
