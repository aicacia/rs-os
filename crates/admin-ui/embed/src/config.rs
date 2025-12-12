use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UIConfig {
  pub server: ServerConfig,
  pub log_level: String,
}

impl Default for UIConfig {
  fn default() -> Self {
    Self {
      server: ServerConfig::default(),
      log_level: "DEBUG".to_owned(),
    }
  }
}

impl<'a> TryFrom<&'a Path> for UIConfig {
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
