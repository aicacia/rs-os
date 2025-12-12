use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use serde::Deserialize;

pub const OIDC_UI_URL_PREFIX: &str = "/oidc";
pub const OIDC_API_URL_PREFIX: &str = "/oidc/api";

pub const ADMIN_UI_URL_PREFIX: &str = "/admin";
pub const ADMIN_API_URL_PREFIX: &str = "/admin/api";

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
  pub oidc: os_oidc::core::config::AppConfig,
  pub admin: os_admin::core::config::AppConfig,
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
      oidc: Default::default(),
      admin: Default::default(),
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

    // OIDC Config Adjustments
    app_config.oidc.server.host = app_config.server.host;
    app_config.oidc.server.port = app_config.server.port;
    app_config.oidc.server.gzip = app_config.server.gzip;

    app_config.oidc.database = app_config.database.clone();

    if let Some(url) = &app_config.url {
      if app_config.oidc.api_url.is_none() {
        app_config.oidc.api_url = Some(format!("{}{}", url, OIDC_API_URL_PREFIX));
      }
      if app_config.oidc.ui_url.is_none() {
        app_config.oidc.ui_url = Some(format!("{}{}", url, OIDC_UI_URL_PREFIX));
      }
    }

    // Admin Config Adjustments
    app_config.admin.server.host = app_config.server.host;
    app_config.admin.server.port = app_config.server.port;
    app_config.admin.server.gzip = app_config.server.gzip;

    app_config.admin.database = app_config.database.clone();

    if let Some(url) = &app_config.url {
      if app_config.admin.api_url.is_none() {
        app_config.admin.api_url = Some(format!("{}{}", url, ADMIN_API_URL_PREFIX));
      }
      if app_config.admin.ui_url.is_none() {
        app_config.admin.ui_url = Some(format!("{}{}", url, ADMIN_UI_URL_PREFIX));
      }
    }

    // Document Store Config Adjustments
    app_config.document_store.server.host = app_config.server.host;
    app_config.document_store.server.port = app_config.server.port;
    app_config.document_store.server.gzip = app_config.server.gzip;

    if let Some(url) = &app_config.url
      && app_config.document_store.api_url.is_none()
    {
      app_config.document_store.api_url = Some(format!("{}{}", url, DOCUMENT_STORE_API_URL_PREFIX));
    }

    // FS Config Adjustments
    app_config.fs.server.host = app_config.server.host;
    app_config.fs.server.port = app_config.server.port;
    app_config.fs.server.gzip = app_config.server.gzip;

    if let Some(url) = &app_config.url
      && app_config.fs.api_url.is_none()
    {
      app_config.fs.api_url = Some(format!("{}{}", url, FS_API_URL_PREFIX));
    }

    Ok(app_config)
  }
}
