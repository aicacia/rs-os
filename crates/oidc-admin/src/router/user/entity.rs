use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use os_oidc_model::entities::users;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
  #[validate(length(min = 1))]
  pub username: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct User {
  pub id: String,
  pub username: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<users::Model> for User {
  fn from(row: users::Model) -> Self {
    Self {
      id: row.id.to_string(),
      username: row.username,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap_or_default(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap_or_default(),
    }
  }
}
