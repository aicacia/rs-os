use std::sync::Arc;

use crate::core::config::app_config::AppConfig;

#[derive(Clone)]
pub struct RouterState {
  pub pool: sqlx::AnyPool,
  pub config: Arc<AppConfig>,
}

unsafe impl Send for RouterState {}
unsafe impl Sync for RouterState {}
