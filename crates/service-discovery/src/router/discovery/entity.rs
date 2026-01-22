use serde::Serialize;
use utoipa::ToSchema;

use crate::config::ServicesConfig;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ServicesDiscovered {
  oidc: String,
  fs_api: String,
  document_store_api: String,
  signaling_api: String,
}

impl<'a> From<&'a ServicesConfig> for ServicesDiscovered {
  fn from(config: &'a ServicesConfig) -> Self {
    Self {
      oidc: config.oidc_api.clone(),
      fs_api: config.fs_api.clone(),
      document_store_api: config.document_store_api.clone(),
      signaling_api: config.signaling_api.clone(),
    }
  }
}
