use std::net::{IpAddr, Ipv4Addr};

use serde::Deserialize;

pub trait AppConfig: Send + Sync + Clone {
  fn base_api_url(&self) -> Result<String, url::ParseError>;
}

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
