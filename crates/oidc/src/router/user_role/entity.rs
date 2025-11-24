use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{model::rbac::sql::RoleSQLRow, router::common::permissions::Permission};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignUserRoleRequest {
  pub role_id: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserRole {
  pub id: String,
  pub uri: String,
  pub description: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<RoleSQLRow> for UserRole {
  fn from(row: RoleSQLRow) -> Self {
    Self {
      id: row.id.to_string(),
      uri: row.uri,
      description: row.description,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap_or_default(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap_or_default(),
    }
  }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserPermissions {
  pub permissions: Vec<Permission>,
}
