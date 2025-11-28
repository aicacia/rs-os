use std::sync::Arc;

/// Generic router state that can hold any configuration type.
/// This allows services to define their own config structures while
/// sharing the same state pattern.
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
