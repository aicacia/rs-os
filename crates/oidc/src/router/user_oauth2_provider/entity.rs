use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::user::orm::UserOAuth2ProviderModel;

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

// Note: This From implementation cannot be used directly because uri and name
// come from the o_auth2_providers table, not user_o_auth2_providers.
// Use the conversion in router code that joins both tables.
impl From<UserOAuth2ProviderModel> for UserOAuth2Provider {
  fn from(row: UserOAuth2ProviderModel) -> Self {
    Self {
      oauth2_provider_id: row.o_auth2_provider_id.to_string(),
      user_id: row.user_id.to_string(),
      uri: String::new(), // Must be populated from o_auth2_providers join
      name: String::new(), // Must be populated from o_auth2_providers join
      email: row.email,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap_or_default(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap_or_default(),
    }
  }
}
