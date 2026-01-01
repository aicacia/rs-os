use std::path::Path;

use os_api::config::ServerConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub log_level: String,
  pub url: Option<String>,
  pub static_dir: String,
  pub env_file: String,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: ServerConfig::default(),
      log_level: "DEBUG".to_owned(),
      url: None,
      static_dir: "./build".to_owned(),
      env_file: "./_app/env.js".to_owned(),
    }
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
