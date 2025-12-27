use std::path::Path;

use os_api::{Environment, ServerConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub log_level: String,
  pub env: Environment,
  pub url: Option<String>,
  pub redis_url: Option<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: ServerConfig::default(),
      log_level: "DEBUG".to_owned(),
      env: Environment::default(),
      url: None,
      redis_url: None,
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
