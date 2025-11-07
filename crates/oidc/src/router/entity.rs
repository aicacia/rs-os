use std::sync::{Arc, atomic::Ordering};

use atomicoption::AtomicValue;

use crate::core::config::{app_config::AppConfig, dynamic_app_config::DynamicAppConfig};

#[derive(Clone)]
pub struct RouterState {
  pub pool: sqlx::AnyPool,
  pub app_config: Arc<AppConfig>,
  pub dynamic_app_config: Arc<AtomicValue<DynamicAppConfig>>,
}

unsafe impl Send for RouterState {}
unsafe impl Sync for RouterState {}

impl RouterState {
  pub fn dynamic_app_config(&self) -> &DynamicAppConfig {
    self.dynamic_app_config.as_ref().as_ref(Ordering::Relaxed)
  }
}
