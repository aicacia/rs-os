use std::fmt::Debug;

use config::Source;
use tokio::{runtime::Handle, task::block_in_place};

use crate::core::config::sql::list_configs;

#[derive(Debug, Clone)]
pub struct ConfigSQLSource {
  pool: sqlx::AnyPool,
}

unsafe impl Send for ConfigSQLSource {}
unsafe impl Sync for ConfigSQLSource {}

impl ConfigSQLSource {
  pub fn new(pool: sqlx::AnyPool) -> Self {
    Self { pool }
  }
}

impl Source for ConfigSQLSource {
  fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
    Box::new(self.clone())
  }

  fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
    let configs = block_in_place(|| {
      Handle::current()
        .block_on(async move { list_configs(&self.pool).await })
        .map_err(|err| config::ConfigError::Message(err.to_string()))
    })?;

    Ok(
      configs
        .into_iter()
        .map(|c| (c.key, c.value.into()))
        .collect(),
    )
  }
}
