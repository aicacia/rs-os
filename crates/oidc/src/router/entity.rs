use std::sync::Arc;

use crate::core::config::app_config::AppConfig;

#[derive(Clone)]
pub struct RouterState {
  pub database: sea_orm::DatabaseConnection,
  pub config: Arc<AppConfig>,
}
