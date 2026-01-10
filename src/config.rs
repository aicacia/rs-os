use std::path::Path;

use serde::Deserialize;

pub const OIDC_UI_URL_PREFIX: &str = "/oidc";
pub const OIDC_API_URL_PREFIX: &str = "/oidc/api";

pub const OIDC_ADMIN_UI_URL_PREFIX: &str = "/oidc-admin";
pub const OIDC_ADMIN_API_URL_PREFIX: &str = "/oidc-admin/api";

pub const FS_API_URL_PREFIX: &str = "/fs/api";

pub const SIGNALING_API_URL_PREFIX: &str = "/signaling/api";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
  pub server: os_api::config::ServerConfig,
  pub database: os_oidc_model::DatabaseConfig,
  pub oidc_api: os_oidc::config::AppConfig,
  pub oidc_ui: os_ui::config::AppConfig,
  pub oidc_admin_api: os_oidc_admin::config::AppConfig,
  pub oidc_admin_ui: os_ui::config::AppConfig,
  pub fs_api: os_fs::config::AppConfig,
  pub service_discovery_api: os_service_discovery::config::AppConfig,
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
      service_discovery_api: Default::default(),
      signaling_api: Default::default(),
      log_level: "DEBUG".to_owned(),
      url: "http://localhost:3000".to_owned(),
      ui_url: None,
    }
  }
}

impl AppConfig {
  pub fn overwrite_dependencies(&mut self) {
    // OIDC UI Config Adjustments
    self.oidc_ui.server = self.server.clone();
    self.oidc_ui.log_level = self.log_level.clone();

    // if oidc_ui url is not set and ui_url is set, use ui_url + prefix
    if self.oidc_ui.url.is_none()
      && let Some(ui_url) = &self.ui_url
    {
      self.oidc_ui.url = Some(format!("{}{}", ui_url, OIDC_UI_URL_PREFIX));
    }
    // if oidc_ui url is still not set, use url + prefix
    if self.oidc_ui.url.is_none() {
      self.oidc_ui.url = Some(format!("{}{}", self.url, OIDC_UI_URL_PREFIX));
    }

    // OIDC API Config Adjustments
    self.oidc_api.server = self.server.clone();
    self.oidc_api.log_level = self.log_level.clone();

    self.oidc_api.database = self.database.clone();

    // if oidc_api url is not set, use url + prefix
    if self.oidc_api.url.is_none() {
      self.oidc_api.url = Some(format!("{}{}", self.url, OIDC_API_URL_PREFIX));
    }
    // if oidc_api ui_url is not set, use oidc_ui url
    if self.oidc_api.ui_url.is_none() {
      self.oidc_api.ui_url = self.oidc_ui.url.clone();
    }

    // Admin UI Config Adjustments
    self.oidc_admin_ui.server = self.server.clone();
    self.oidc_admin_ui.log_level = self.log_level.clone();

    // if oidc_admin_ui url is not set and ui_url is set, use ui_url + prefix
    if self.oidc_admin_ui.url.is_none()
      && let Some(ui_url) = &self.ui_url
    {
      self.oidc_admin_ui.url = Some(format!("{}{}", ui_url, OIDC_ADMIN_UI_URL_PREFIX));
    }
    // if oidc_admin_ui url is still not set, use url + prefix
    if self.oidc_admin_ui.url.is_none() {
      self.oidc_admin_ui.url = Some(format!("{}{}", self.url, OIDC_ADMIN_UI_URL_PREFIX));
    }

    // Admin API Config Adjustments
    self.oidc_admin_api.server = self.server.clone();
    self.oidc_admin_api.log_level = self.log_level.clone();

    self.oidc_admin_api.database = self.database.clone();

    // if oidc_admin_api url is not set, use url + prefix
    if self.oidc_admin_api.url.is_none() {
      self.oidc_admin_api.url = Some(format!("{}{}", self.url, OIDC_ADMIN_API_URL_PREFIX));
    }
    // if oidc_admin_api ui_url is not set, use oidc_admin_ui url
    if self.oidc_admin_api.ui_url.is_none() {
      self.oidc_admin_api.ui_url = self.oidc_admin_ui.url.clone();
    }

    // FS API Config Adjustments
    self.fs_api.server = self.server.clone();
    self.fs_api.log_level = self.log_level.clone();

    if self.fs_api.url.is_none() {
      self.fs_api.url = Some(format!("{}{}", self.url, FS_API_URL_PREFIX));
    }

    // Signaling API Config Adjustments
    self.signaling_api.server = self.server.clone();
    self.signaling_api.log_level = self.log_level.clone();

    if self.signaling_api.url.is_none() {
      self.signaling_api.url = Some(format!("{}{}", self.url, SIGNALING_API_URL_PREFIX));
    }

    self.service_discovery_api.services.fs_api = self.fs_api.url.clone();
    self.service_discovery_api.services.oidc_api = self.oidc_api.url();
    self.service_discovery_api.services.oidc_ui = self.oidc_ui.url();
    self.service_discovery_api.services.oidc_admin_api = Some(self.oidc_admin_api.url());
    self.service_discovery_api.services.oidc_admin_ui = Some(self.oidc_admin_ui.url());
    self.service_discovery_api.services.signaling_api = Some(self.signaling_api.url());
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
