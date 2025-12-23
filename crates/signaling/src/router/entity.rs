use std::sync::Arc;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct RouterState {
  pub config: Arc<AppConfig>,
  pub redis_client: redis::Client,
}
