use serde::Serialize;
use utoipa::ToSchema;

use crate::config::ServicesConfig;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ServicesDiscovered {
  oidc: String,
  fs: Option<String>,
  document_store: Option<String>,
  signaling: Option<String>,
}

impl<'a> From<&'a ServicesConfig> for ServicesDiscovered {
  fn from(config: &'a ServicesConfig) -> Self {
    Self {
      fs: config.fs_api.clone(),
      oidc: config.oidc_api.clone(),
      document_store: config.document_store_api.clone(),
      signaling: config.signaling_api.clone(),
    }
  }
}
