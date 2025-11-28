use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ServerConfig {
  pub host: IpAddr,
  pub port: u16,
  pub gzip: bool,
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      host: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
      port: 3000,
      gzip: true,
    }
  }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
  #[default]
  Local,
  Development,
  Production,
}

impl Environment {
  pub fn as_str(&self) -> &'static str {
    match self {
      Environment::Local => "local",
      Environment::Development => "development",
      Environment::Production => "production",
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub log_level: String,
  pub env: Environment,
  pub api_url: Option<String>,
  pub ui_url: Option<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: ServerConfig::default(),
      log_level: tracing::Level::DEBUG.to_string(),
      env: Environment::default(),
      api_url: None,
      ui_url: None,
    }
  }
}

impl AppConfig {
  pub fn api_url(&self) -> String {
    self
      .api_url
      .to_owned()
      .unwrap_or_else(|| format!("http://{}:{}", self.server.host, self.server.port))
  }

  pub fn base_api_url(&self) -> Result<String, url::ParseError> {
    let url = url::Url::parse(&self.api_url())?;
    Ok(url.origin().unicode_serialization())
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
