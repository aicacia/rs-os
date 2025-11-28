use std::sync::Arc;

#[derive(Clone)]
pub struct RouterState<C> {
  pub config: Arc<C>,
}

impl<C> RouterState<C> {
  pub fn new(config: C) -> Self {
    Self {
      config: Arc::new(config),
    }
  }

  pub fn from_arc(config: Arc<C>) -> Self {
    Self { config }
  }
}
