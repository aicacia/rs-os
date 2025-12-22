use std::net::{IpAddr, Ipv4Addr};

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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
  Local,
  Development,
  Production,
}

impl Default for Environment {
  fn default() -> Self {
    match std::env::var("APP_ENV") {
      Ok(env) => match env.as_str() {
        "local" => Environment::Local,
        "development" => Environment::Development,
        "production" => Environment::Production,
        _ => Environment::Local,
      },
      Err(_) => Environment::Local,
    }
  }
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
