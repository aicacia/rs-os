use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use os_db::database_config::DatabaseConfig;
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
pub struct PasswordConfig {
  pub salt_length: usize,
  pub hash_length: u32,
  pub memory_mib: u32,
  pub iterations: u32,
  pub parallelism: u32,
  pub history: u8,
  pub expire_days: u8,
}

impl Default for PasswordConfig {
  fn default() -> Self {
    Self {
      salt_length: 16,
      hash_length: 32,
      memory_mib: 19,
      iterations: 2,
      parallelism: 1,
      history: 24,
      expire_days: 60,
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct UserConfig {
  pub register_enabled: bool,
  pub allow_passwords: bool,
}

impl Default for UserConfig {
  fn default() -> Self {
    Self {
      register_enabled: false,
      allow_passwords: true,
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct OAuth2 {
  pub register_enabled: bool,
  pub code_timeout_in_seconds: u64,
}

impl Default for OAuth2 {
  fn default() -> Self {
    Self {
      register_enabled: false,
      code_timeout_in_seconds: 60 * 5,
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TokenConfig {
  pub expires_in_seconds: u64,
  pub refresh_expires_in_seconds: u64,
}

impl Default for TokenConfig {
  fn default() -> Self {
    Self {
      expires_in_seconds: 86400,          // 1 day
      refresh_expires_in_seconds: 604800, // 1 week
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub log_level: String,
  pub env: Environment,
  pub password: PasswordConfig,
  pub user: UserConfig,
  pub oauth2: OAuth2,
  pub token: TokenConfig,
  pub public_url: Option<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: ServerConfig::default(),
      database: DatabaseConfig::default(),
      log_level: tracing::Level::DEBUG.to_string(),
      env: Environment::default(),
      password: PasswordConfig::default(),
      user: UserConfig::default(),
      oauth2: OAuth2::default(),
      token: TokenConfig::default(),
      public_url: None,
    }
  }
}

impl AppConfig {
  pub fn public_url(&self) -> String {
    self
      .public_url
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
