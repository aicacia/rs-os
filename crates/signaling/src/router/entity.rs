use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{config::AppConfig, router::ws::pubsub::PubSub};

#[derive(Clone)]
pub struct RouterState {
  pub config: Arc<AppConfig>,
  pub pubsub: Arc<PubSub>,
  pub cancellation_token: CancellationToken,
}
