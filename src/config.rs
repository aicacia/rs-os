use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use serde::Deserialize;

pub const OIDC_UI_URL_PREFIX: &str = "/oidc";
pub const OIDC_API_URL_PREFIX: &str = "/oidc/api";

pub const OIDC_ADMIN_UI_URL_PREFIX: &str = "/oidc-admin";
pub const OIDC_ADMIN_API_URL_PREFIX: &str = "/oidc-admin/api";

pub const SIGNALING_API_URL_PREFIX: &str = "/signaling/api";

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
pub struct AppConfig {
  pub server: ServerConfig,
  pub database: os_oidc_model::DatabaseConfig,
  pub oidc_api: os_oidc::config::AppConfig,
  pub oidc_ui: os_ui::config::AppConfig,
  pub oidc_admin_api: os_oidc_admin::config::AppConfig,
  pub oidc_admin_ui: os_ui::config::AppConfig,
  pub signaling_api: os_signaling::config::AppConfig,
  pub log_level: String,
  pub url: Option<String>,
  pub ui_url: Option<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: Default::default(),
      database: Default::default(),
      oidc_api: Default::default(),
      oidc_ui: Default::default(),
      oidc_admin_api: Default::default(),
      oidc_admin_ui: Default::default(),
      signaling_api: Default::default(),
      log_level: "DEBUG".to_owned(),
      url: None,
      ui_url: None,
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

    // OIDC API Config Adjustments
    app_config.oidc_api.server.host = app_config.server.host;
    app_config.oidc_api.server.port = app_config.server.port;
    app_config.oidc_api.server.gzip = app_config.server.gzip;
    app_config.oidc_api.log_level = app_config.log_level.clone();

    app_config.oidc_api.database = app_config.database.clone();

    if let Some(ui_url) = &app_config.ui_url
      && app_config.oidc_api.ui_url.is_none()
    {
      app_config.oidc_api.ui_url = Some(format!("{}{}", ui_url, OIDC_UI_URL_PREFIX));
    }
    if let Some(url) = &app_config.url {
      if app_config.oidc_api.url.is_none() {
        app_config.oidc_api.url = Some(format!("{}{}", url, OIDC_API_URL_PREFIX));
      }
      if app_config.oidc_api.ui_url.is_none() {
        app_config.oidc_api.ui_url = Some(format!("{}{}", url, OIDC_UI_URL_PREFIX));
        app_config.oidc_ui.url = Some(format!("{}{}", url, OIDC_UI_URL_PREFIX));
      }
    }

    // OIDC UI Config Adjustments
    app_config.oidc_ui.server.host = app_config.server.host;
    app_config.oidc_ui.server.port = app_config.server.port;
    app_config.oidc_ui.server.gzip = app_config.server.gzip;
    app_config.oidc_ui.log_level = app_config.log_level.clone();

    // Admin API Config Adjustments
    app_config.oidc_admin_api.server.host = app_config.server.host;
    app_config.oidc_admin_api.server.port = app_config.server.port;
    app_config.oidc_admin_api.server.gzip = app_config.server.gzip;
    app_config.oidc_admin_api.log_level = app_config.log_level.clone();

    app_config.oidc_admin_api.database = app_config.database.clone();

    if let Some(ui_url) = &app_config.ui_url
      && app_config.oidc_admin_api.ui_url.is_none()
    {
      app_config.oidc_admin_api.ui_url = Some(format!("{}{}", ui_url, OIDC_ADMIN_UI_URL_PREFIX));
    }
    if let Some(url) = &app_config.url {
      if app_config.oidc_admin_api.url.is_none() {
        app_config.oidc_admin_api.url = Some(format!("{}{}", url, OIDC_ADMIN_API_URL_PREFIX));
      }
      if app_config.oidc_admin_api.ui_url.is_none() {
        app_config.oidc_admin_api.ui_url = Some(format!("{}{}", url, OIDC_ADMIN_UI_URL_PREFIX));
        app_config.oidc_admin_ui.url = Some(format!("{}{}", url, OIDC_ADMIN_UI_URL_PREFIX));
      }
    }

    // Admin UI Config Adjustments
    app_config.oidc_admin_ui.server.host = app_config.server.host;
    app_config.oidc_admin_ui.server.port = app_config.server.port;
    app_config.oidc_admin_ui.server.gzip = app_config.server.gzip;
    app_config.oidc_admin_ui.log_level = app_config.log_level.clone();

    // Signaling API Config Adjustments
    app_config.signaling_api.server.host = app_config.server.host;
    app_config.signaling_api.server.port = app_config.server.port;
    app_config.signaling_api.server.gzip = app_config.server.gzip;
    app_config.signaling_api.log_level = app_config.log_level.clone();

    if let Some(url) = &app_config.url
      && app_config.signaling_api.url.is_none()
    {
      app_config.signaling_api.url = Some(format!("{}{}", url, SIGNALING_API_URL_PREFIX));
    }

    Ok(app_config)
  }
}
