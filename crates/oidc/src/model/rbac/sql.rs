#[derive(sqlx::FromRow)]
pub struct RoleSQLRow {
  pub id: i64,
  pub uri: String,
  pub description: String,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(sqlx::FromRow)]
pub struct PermissionSQLRow {
  pub id: i64,
  pub uri: String,
  pub description: String,
  pub updated_at: i64,
  pub created_at: i64,
}

impl From<RolePermissionSQLRow> for PermissionSQLRow {
  fn from(value: RolePermissionSQLRow) -> Self {
    Self {
      id: value.id,
      uri: value.uri,
      description: value.description,
      updated_at: value.updated_at,
      created_at: value.created_at,
    }
  }
}

#[derive(sqlx::FromRow)]
pub struct RolePermissionSQLRow {
  pub id: i64,
  pub role_id: i64,
  pub uri: String,
  pub description: String,
  pub updated_at: i64,
  pub created_at: i64,
}

pub async fn get_role_permissions_by_role_id(
  pool: &sqlx::AnyPool,
  role_id: i64,
) -> sqlx::Result<Vec<PermissionSQLRow>> {
  sqlx::query_as(
    r#"SELECT p.*
    FROM roles_permissions rp
    JOIN permissions p ON p.id = rp.permission_id
    WHERE rp.role_id = $1;"#,
  )
  .bind(role_id)
  .fetch_all(pool)
  .await
}
