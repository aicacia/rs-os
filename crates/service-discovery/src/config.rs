use std::path::Path;

use os_api::{Environment, ServerConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ServicesConfig {
  pub fs_api: Option<String>,
  pub oidc_api: String,
  pub oidc_ui: String,
  pub oidc_admin_api: Option<String>,
  pub oidc_admin_ui: Option<String>,
  pub signaling_api: Option<String>,
}

impl Default for ServicesConfig {
  fn default() -> Self {
    Self {
      fs_api: None,
      oidc_api: "http://localhost:3000/oidc/api".to_owned(),
      oidc_ui: "http://localhost:3000/oidc".to_owned(),
      oidc_admin_api: None,
      oidc_admin_ui: None,
      signaling_api: None,
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub services: ServicesConfig,
  pub log_level: String,
  pub env: Environment,
  pub url: Option<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: ServerConfig::default(),
      services: ServicesConfig::default(),
      log_level: "DEBUG".to_owned(),
      env: Environment::default(),
      url: None,
    }
  }
}

impl AppConfig {
  pub fn url(&self) -> String {
    self
      .url
      .to_owned()
      .unwrap_or_else(|| format!("http://{}:{}", self.server.host, self.server.port))
  }

  pub fn base_url(&self) -> Result<String, url::ParseError> {
    let url = url::Url::parse(self.url().as_ref())?;
    Ok(url.origin().unicode_serialization())
  }
}

impl<'a> TryFrom<&'a Path> for AppConfig {
  type Error = config::ConfigError;

  fn try_from(config_path: &'a Path) -> Result<Self, Self::Error> {
    config::Config::builder()
      .add_source(config::File::with_name(
        config_path.to_string_lossy().as_ref(),
      ))
      .add_source(config::Environment::with_prefix("APP"))
      .build()?
      .try_deserialize()
  }
}
