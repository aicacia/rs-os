use std::sync::Arc;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct RouterState {
  pub database_connection: sea_orm::DatabaseConnection,
  pub config: Arc<AppConfig>,
}
