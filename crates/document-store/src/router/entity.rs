use std::sync::Arc;

use crate::core::config::app_config::AppConfig;

#[derive(Clone)]
pub struct RouterState {
  pub config: Arc<AppConfig>,
}
