use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::user::sql::UserPhoneNumberSQLRow;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserPhoneNumberRequest {
  pub phone_number: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserPhoneNumberRequest {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_primary: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserPhoneNumber {
  pub id: String,
  pub user_id: String,
  pub phone_number: String,
  pub verified: bool,
  pub is_primary: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<UserPhoneNumberSQLRow> for UserPhoneNumber {
  fn from(row: UserPhoneNumberSQLRow) -> Self {
    let verified = row.is_verified();
    let is_primary = row.is_primary();
    Self {
      id: row.id.to_string(),
      user_id: row.user_id.to_string(),
      phone_number: row.phone_number,
      verified,
      is_primary,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap_or_default(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap_or_default(),
    }
  }
}
