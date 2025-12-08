use std::path::Path;

use os_api::{Environment, ServerConfig};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
pub enum StorageAdapterKind {
  #[default]
  Sled,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct StorageConfig {
  pub storage_adapter: StorageAdapterKind,
  pub data_path: String,
}

impl Default for StorageConfig {
  fn default() -> Self {
    Self {
      storage_adapter: StorageAdapterKind::default(),
      data_path: "./storage/data".to_string(),
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub log_level: String,
  pub env: Environment,
  pub storage: StorageConfig,
  pub api_url: Option<String>,
  pub ui_url: Option<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: ServerConfig::default(),
      log_level: tracing::Level::DEBUG.to_string(),
      env: Environment::default(),
      storage: StorageConfig::default(),
      api_url: None,
      ui_url: None,
    }
  }
}

impl os_api::AppConfig for AppConfig {
  fn base_api_url(&self) -> Result<String, url::ParseError> {
    let url = url::Url::parse(&self.api_url())?;
    Ok(url.origin().unicode_serialization())
  }
}

impl AppConfig {
  pub fn api_url(&self) -> String {
    self
      .api_url
      .to_owned()
      .unwrap_or_else(|| format!("http://{}:{}", self.server.host, self.server.port))
  }
}

impl<'a> TryFrom<&'a Path> for AppConfig {
  type Error = config::ConfigError;

  fn try_from(config_path: &'a Path) -> Result<Self, Self::Error> {
    Ok(
      config::Config::builder()
        .add_source(config::File::with_name(
          config_path.to_string_lossy().as_ref(),
        ))
        .add_source(config::Environment::with_prefix("APP"))
        .build()?
        .try_deserialize()?,
    )
  }
}
