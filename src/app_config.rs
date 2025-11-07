use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub oidc: os_oidc::core::config::app_config::AppConfig,
  pub log_level: String,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: Default::default(),
      oidc: Default::default(),
      log_level: "DEBUG".to_owned(),
    }
  }
}

impl<'a> TryFrom<&'a Path> for AppConfig {
  type Error = config::ConfigError;

  fn try_from(config_path: &'a Path) -> Result<Self, Self::Error> {
    let mut app_config: AppConfig = config::Config::builder()
      .add_source(config::File::with_name(
        config_path.to_string_lossy().as_ref(),
      ))
      .add_source(config::Environment::with_prefix("APP"))
      .build()?
      .try_deserialize()?;

    app_config.oidc.server.host = app_config.server.host.clone();
    app_config.oidc.server.port = app_config.server.port;
    app_config.oidc.server.gzip = app_config.server.gzip;

    Ok(app_config)
  }
}
