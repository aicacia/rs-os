use std::path::Path;

use os_api::{Environment, ServerConfig};
use os_model::DatabaseConfig;
use serde::Deserialize;

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
      memory_mib: 47,
      iterations: 3,
      parallelism: 4,
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
  pub default_jwt_algorithm: jsonwebtoken::Algorithm,
  pub expires_in_seconds: u64,
  pub refresh_expires_in_seconds: u64,
}

impl Default for TokenConfig {
  fn default() -> Self {
    Self {
      default_jwt_algorithm: jsonwebtoken::Algorithm::EdDSA,
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
  pub url: Option<String>,
  pub ui_url: Option<String>,
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
      url: None,
      ui_url: None,
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

  pub fn ui_url(&self) -> String {
    self
      .ui_url
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
