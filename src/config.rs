use std::{
  net::{IpAddr, Ipv4Addr},
  path::Path,
};

use serde::Deserialize;

pub const OIDC_UI_URL_PREFIX: &str = "/oidc";
pub const OIDC_API_URL_PREFIX: &str = "/oidc/api";

pub const OIDC_ADMIN_UI_URL_PREFIX: &str = "/oidc-admin";
pub const OIDC_ADMIN_API_URL_PREFIX: &str = "/oidc-admin/api";

pub const FS_API_URL_PREFIX: &str = "/fs/api";

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
  pub fs_api: os_fs::config::AppConfig,
  pub signaling_api: os_signaling::config::AppConfig,
  pub log_level: String,
  pub url: String,
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
      fs_api: Default::default(),
      signaling_api: Default::default(),
      log_level: "DEBUG".to_owned(),
      url: "http://localhost:3000".to_owned(),
      ui_url: None,
    }
  }
}

impl AppConfig {
  pub fn overwrite_dependencies(&mut self) {
    // OIDC API Config Adjustments
    self.oidc_api.server.host = self.server.host;
    self.oidc_api.server.port = self.server.port;
    self.oidc_api.server.gzip = self.server.gzip;
    self.oidc_api.log_level = self.log_level.clone();

    self.oidc_api.database = self.database.clone();

    if let Some(ui_url) = &self.ui_url
      && self.oidc_api.ui_url.is_none()
    {
      self.oidc_api.ui_url = Some(format!("{}{}", ui_url, OIDC_UI_URL_PREFIX));
    }
    if self.oidc_api.url.is_none() {
      self.oidc_api.url = Some(format!("{}{}", self.url, OIDC_API_URL_PREFIX));
    }
    if self.oidc_api.ui_url.is_none() {
      self.oidc_api.ui_url = Some(format!("{}{}", self.url, OIDC_UI_URL_PREFIX));
      self.oidc_ui.url = Some(format!("{}{}", self.url, OIDC_UI_URL_PREFIX));
    }

    // OIDC UI Config Adjustments
    self.oidc_ui.server.host = self.server.host;
    self.oidc_ui.server.port = self.server.port;
    self.oidc_ui.server.gzip = self.server.gzip;
    self.oidc_ui.log_level = self.log_level.clone();

    // Admin API Config Adjustments
    self.oidc_admin_api.server.host = self.server.host;
    self.oidc_admin_api.server.port = self.server.port;
    self.oidc_admin_api.server.gzip = self.server.gzip;
    self.oidc_admin_api.log_level = self.log_level.clone();

    self.oidc_admin_api.database = self.database.clone();

    if let Some(ui_url) = &self.ui_url
      && self.oidc_admin_api.ui_url.is_none()
    {
      self.oidc_admin_api.ui_url = Some(format!("{}{}", ui_url, OIDC_ADMIN_UI_URL_PREFIX));
    }
    if self.oidc_admin_api.url.is_none() {
      self.oidc_admin_api.url = Some(format!("{}{}", self.url, OIDC_ADMIN_API_URL_PREFIX));
    }
    if self.oidc_admin_api.ui_url.is_none() {
      self.oidc_admin_api.ui_url = Some(format!("{}{}", self.url, OIDC_ADMIN_UI_URL_PREFIX));
      self.oidc_admin_ui.url = Some(format!("{}{}", self.url, OIDC_ADMIN_UI_URL_PREFIX));
    }

    // Admin UI Config Adjustments
    self.oidc_admin_ui.server.host = self.server.host;
    self.oidc_admin_ui.server.port = self.server.port;
    self.oidc_admin_ui.server.gzip = self.server.gzip;
    self.oidc_admin_ui.log_level = self.log_level.clone();

    // FS API Config Adjustments
    self.fs_api.server.host = self.server.host;
    self.fs_api.server.port = self.server.port;
    self.fs_api.server.gzip = self.server.gzip;
    self.fs_api.log_level = self.log_level.clone();

    if self.fs_api.url.is_none() {
      self.fs_api.url = Some(format!("{}{}", self.url, FS_API_URL_PREFIX));
    }

    // Signaling API Config Adjustments
    self.signaling_api.server.host = self.server.host;
    self.signaling_api.server.port = self.server.port;
    self.signaling_api.server.gzip = self.server.gzip;
    self.signaling_api.log_level = self.log_level.clone();

    if self.signaling_api.url.is_none() {
      self.signaling_api.url = Some(format!("{}{}", self.url, SIGNALING_API_URL_PREFIX));
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

    app_config.overwrite_dependencies();

    Ok(app_config)
  }
}
