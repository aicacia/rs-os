use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct RouterState {
  pub cancellation_token: CancellationToken,
  pub config: Arc<AppConfig>,
}
