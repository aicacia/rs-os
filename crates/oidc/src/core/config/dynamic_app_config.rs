use std::{
  sync::{Arc, atomic::Ordering},
  time::Duration,
};

use atomicoption::AtomicValue;
use serde::Deserialize;
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::core::config::source::ConfigSQLSource;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DynamicAppConfig {
  pub password: PasswordConfig,
  pub user: UserConfig,
  pub oauth2: OAuth2,
  pub token: TokenConfig,
  pub reload_config_interval_in_seconds: u64,
  pub public_url: Option<String>,
}

impl Default for DynamicAppConfig {
  fn default() -> Self {
    Self {
      password: PasswordConfig::default(),
      user: UserConfig::default(),
      oauth2: OAuth2::default(),
      token: TokenConfig::default(),
      reload_config_interval_in_seconds: 60 * 5,
      public_url: None,
    }
  }
}

impl DynamicAppConfig {
  pub fn new(pool: sqlx::AnyPool) -> Result<Self, config::ConfigError> {
    Ok(
      config::Config::builder()
        .add_source(ConfigSQLSource::new(pool))
        .build()?
        .try_deserialize()?,
    )
  }

  pub fn with_background_updater(
    pool: sqlx::AnyPool,
    cancellation_token: CancellationToken,
  ) -> Result<Arc<AtomicValue<DynamicAppConfig>>, config::ConfigError> {
    let initial_config = Self::new(pool.clone())?;
    let reload_config_interval_in_seconds = initial_config.reload_config_interval_in_seconds;

    let shared = Arc::new(AtomicValue::new(initial_config));
    let background = shared.clone();

    tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_secs(reload_config_interval_in_seconds));

      loop {
        let current_reload_config_interval_in_seconds = background
          .as_ref()
          .as_ref(Ordering::Relaxed)
          .reload_config_interval_in_seconds;

        tokio::select! {
          _ = interval.tick() => {
            match Self::new(pool.clone()) {
              Ok(new_config) => {
                if current_reload_config_interval_in_seconds != new_config.reload_config_interval_in_seconds {
                  interval = time::interval(Duration::from_secs(new_config.reload_config_interval_in_seconds));
                }
                background.store(Ordering::Relaxed, new_config);
              }
              Err(err) => {
                log::error!("Failed to reload DynamicAppConfig: {err}");
              }
            }
          }
          _ = cancellation_token.cancelled() => {
            break;
          }
        }
      }
    });

    Ok(shared)
  }
}
