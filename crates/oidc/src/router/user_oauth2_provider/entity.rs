use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::user::sql::UserOAuth2ProviderSQLRow;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkUserOAuth2ProviderRequest {
  pub provider_id: i64,
  pub name: String,
  pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserOAuth2Provider {
  pub oauth2_provider_id: String,
  pub user_id: String,
  pub uri: String,
  pub name: String,
  pub email: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<UserOAuth2ProviderSQLRow> for UserOAuth2Provider {
  fn from(row: UserOAuth2ProviderSQLRow) -> Self {
    Self {
      oauth2_provider_id: row.oauth2_provider_id.to_string(),
      user_id: row.user_id.to_string(),
      uri: row.uri,
      name: row.name,
      email: row.email,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap_or_default(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap_or_default(),
    }
  }
}
