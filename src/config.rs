use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use serde::Deserialize;

pub const OIDC_UI_URL_PREFIX: &str = "/oidc";
pub const OIDC_API_URL_PREFIX: &str = "/oidc/api";

pub const ADMIN_UI_URL_PREFIX: &str = "/admin";

pub const DOCUMENT_STORE_API_URL_PREFIX: &str = "/document-store/api";

pub const FS_API_URL_PREFIX: &str = "/fs/api";

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
  pub database: os_model::DatabaseConfig,
  pub oidc_api: os_oidc::core::config::AppConfig,
  pub oidc_ui: os_oidc_ui_embed::config::AppConfig,
  pub admin_ui: os_admin_ui_embed::config::AppConfig,
  pub document_store: os_document_store::core::config::AppConfig,
  pub fs: os_fs::core::config::AppConfig,
  pub log_level: String,
  pub url: Option<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      server: Default::default(),
      database: Default::default(),
      oidc_api: Default::default(),
      oidc_ui: Default::default(),
      admin_ui: Default::default(),
      document_store: Default::default(),
      fs: Default::default(),
      log_level: "DEBUG".to_owned(),
      url: None,
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

    if let Some(url) = &app_config.url {
      if app_config.oidc_api.api_url.is_none() {
        app_config.oidc_api.api_url = Some(format!("{}{}", url, OIDC_API_URL_PREFIX));
      }
      if app_config.oidc_api.ui_url.is_none() {
        app_config.oidc_api.ui_url = Some(format!("{}{}", url, OIDC_UI_URL_PREFIX));
      }
    }

    // OIDC UI Config Adjustments
    app_config.oidc_ui.server.host = app_config.server.host;
    app_config.oidc_ui.server.port = app_config.server.port;
    app_config.oidc_ui.server.gzip = app_config.server.gzip;
    app_config.oidc_ui.log_level = app_config.log_level.clone();

    // Document Store Config Adjustments
    app_config.document_store.server.host = app_config.server.host;
    app_config.document_store.server.port = app_config.server.port;
    app_config.document_store.server.gzip = app_config.server.gzip;
    app_config.document_store.log_level = app_config.log_level.clone();

    if let Some(url) = &app_config.url
      && app_config.document_store.api_url.is_none()
    {
      app_config.document_store.api_url = Some(format!("{}{}", url, DOCUMENT_STORE_API_URL_PREFIX));
    }

    // FS Config Adjustments
    app_config.fs.server.host = app_config.server.host;
    app_config.fs.server.port = app_config.server.port;
    app_config.fs.server.gzip = app_config.server.gzip;
    app_config.fs.log_level = app_config.log_level.clone();

    if let Some(url) = &app_config.url
      && app_config.fs.api_url.is_none()
    {
      app_config.fs.api_url = Some(format!("{}{}", url, FS_API_URL_PREFIX));
    }

    Ok(app_config)
  }
}
