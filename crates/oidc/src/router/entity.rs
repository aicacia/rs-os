use std::sync::Arc;

use crate::core::config::app_config::AppConfig;

/// OIDC-specific router state that includes database pool
#[derive(Clone)]
pub struct RouterState {
  pub pool: sqlx::AnyPool,
  pub config: Arc<AppConfig>,
}
