use crate::core::config::{app_config::AppConfig, dynamic_app_config::DynamicAppConfig};

pub fn public_url(app_config: &AppConfig, dynamic_app_config: &DynamicAppConfig) -> String {
  dynamic_app_config.public_url.to_owned().unwrap_or_else(|| {
    format!(
      "http://{}:{}",
      app_config.server.host, app_config.server.port
    )
  })
}
