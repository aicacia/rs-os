use std::{io, pin::Pin};

use futures_util::stream::Stream;
use tokio_util::sync::CancellationToken;

use crate::router::ws::pubsub::{InMemoryPubSub, RedisPubSub};

pub type MessageStream = Pin<Box<dyn Stream<Item = String> + Send>>;

#[async_trait::async_trait]
pub trait PubSubAdapterInternal: Send + Sync {
  async fn get_peers(&self, room: &str) -> io::Result<Vec<String>>;
  async fn add_user(&self, room: &str, user_id: &str) -> io::Result<()>;
  async fn remove_user(&self, room: &str, user_id: &str) -> io::Result<()>;
  async fn broadcast(&self, room: &str, payload: &str) -> io::Result<()>;
  async fn send(&self, room: &str, user_id: &str, payload: &str) -> io::Result<()>;
  async fn subscribe(
    &self,
    room: &str,
    user_id: &str,
    cancellation_token: CancellationToken,
  ) -> io::Result<MessageStream>;
}

pub struct PubSub {
  inner: Box<dyn PubSubAdapterInternal>,
}

impl PubSub {
  pub fn new_redis(redis_url: &str) -> Result<Self, redis::RedisError> {
    Ok(Self {
      inner: Box::new(RedisPubSub::new(redis::Client::open(redis_url)?)),
    })
  }
}

impl PubSub {
  pub fn new_in_memory() -> Self {
    Self {
      inner: Box::new(InMemoryPubSub::new()),
    }
  }
}

impl PubSub {
  pub async fn get_peers(&self, room: &str) -> io::Result<Vec<String>> {
    self.inner.get_peers(room).await
  }

  pub async fn add_user(&self, room: &str, user_id: &str) -> io::Result<()> {
    self.inner.add_user(room, user_id).await
  }

  pub async fn remove_user(&self, room: &str, user_id: &str) -> io::Result<()> {
    self.inner.remove_user(room, user_id).await
  }

  pub async fn broadcast(&self, room: &str, payload: &str) -> io::Result<()> {
    self.inner.broadcast(room, payload).await
  }

  pub async fn send(&self, room: &str, user_id: &str, payload: &str) -> io::Result<()> {
    self.inner.send(room, user_id, payload).await
  }

  pub async fn subscribe(
    &self,
    room: &str,
    user_id: &str,
    cancellation_token: CancellationToken,
  ) -> io::Result<MessageStream> {
    self
      .inner
      .subscribe(room, user_id, cancellation_token)
      .await
  }
}
