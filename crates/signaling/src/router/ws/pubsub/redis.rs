use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Stream, StreamExt, stream};
use redis::AsyncCommands;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::{MessageStream, pubsub::PubSubAdapterInternal};

struct GuardedStream<S> {
  stream: Pin<Box<S>>,
  handle: JoinHandle<()>,
}

impl<S: Stream> Stream for GuardedStream<S> {
  type Item = S::Item;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.stream.as_mut().poll_next(cx)
  }
}

impl<S> Drop for GuardedStream<S> {
  fn drop(&mut self) {
    self.handle.abort();
  }
}

#[derive(Clone)]
pub struct RedisPubSub {
  client: redis::Client,
}

impl RedisPubSub {
  pub fn new(client: redis::Client) -> Self {
    Self { client }
  }
}

#[async_trait::async_trait]
impl PubSubAdapterInternal for RedisPubSub {
  async fn get_peers(&self, room: &str) -> io::Result<Vec<String>> {
    let client = self.client.clone();
    let room = room.to_owned();

    let users_key = format!("{room}:users");

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| io::Error::other(format!("Failed to get Redis connection: {}", e)))?;

    let users: Vec<String> = conn
      .smembers::<_, Vec<String>>(users_key)
      .await
      .map_err(|e| io::Error::other(format!("Failed to get room users: {}", e)))?;

    Ok(users)
  }

  async fn add_user(&self, room: &str, user_id: &str) -> io::Result<()> {
    let client = self.client.clone();
    let room = room.to_owned();
    let user_id = user_id.to_owned();

    let users_key = format!("{room}:users");

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| io::Error::other(format!("Failed to get Redis connection: {}", e)))?;

    let _: usize = conn
      .sadd(&users_key, &user_id)
      .await
      .map_err(|e| io::Error::other(format!("Failed to add user to room: {}", e)))?;

    Ok(())
  }

  async fn remove_user(&self, room: &str, user_id: &str) -> io::Result<()> {
    let client = self.client.clone();
    let room = room.to_owned();
    let user_id = user_id.to_owned();

    let users_key = format!("{room}:users");

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| io::Error::other(format!("Failed to get Redis connection: {}", e)))?;

    conn
      .srem::<_, _, ()>(&users_key, &user_id)
      .await
      .map_err(|e| io::Error::other(format!("Failed to remove user from room: {}", e)))?;

    let remaining_users: usize = conn
      .scard(&users_key)
      .await
      .map_err(|e| io::Error::other(format!("Failed to get room user count: {}", e)))?;

    if remaining_users == 0 {
      let _: usize = conn
        .del(&users_key)
        .await
        .map_err(|e| io::Error::other(format!("Failed to delete empty room: {}", e)))?;
    }

    Ok(())
  }

  async fn broadcast(&self, room: &str, payload: &str) -> io::Result<()> {
    let client = self.client.clone();
    let room = room.to_owned();
    let payload = payload.to_owned();

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| io::Error::other(format!("Failed to get Redis connection: {}", e)))?;

    conn
      .publish::<_, _, ()>(&room, payload)
      .await
      .map_err(|e| io::Error::other(format!("Failed to publish message: {}", e)))
  }

  async fn send(&self, room: &str, user_id: &str, payload: &str) -> io::Result<()> {
    let client = self.client.clone();
    let room = room.to_owned();
    let user_id = user_id.to_owned();
    let payload = payload.to_owned();

    let users_key = format!("{room}:users");

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| io::Error::other(format!("Failed to get Redis connection: {}", e)))?;

    let is_member: bool = conn
      .sismember(&users_key, &user_id)
      .await
      .map_err(|e| io::Error::other(format!("Failed to check user membership: {}", e)))?;

    if is_member {
      let user_channel = format!("{room}:{user_id}");
      conn
        .publish::<_, _, ()>(&user_channel, payload)
        .await
        .map_err(|e| io::Error::other(format!("Failed to send message to user: {}", e)))?;
    }

    Ok(())
  }

  async fn subscribe(
    &self,
    room: &str,
    user_id: &str,
    cancellation_token: CancellationToken,
  ) -> io::Result<MessageStream> {
    let client = self.client.clone();
    let room = room.to_owned();
    let user_id = user_id.to_owned();

    let mut pubsub = client
      .get_async_pubsub()
      .await
      .map_err(|e| io::Error::other(format!("Failed to get Redis pubsub connection: {}", e)))?;

    pubsub
      .subscribe(&room)
      .await
      .map_err(|e| io::Error::other(format!("Failed to subscribe to room channel: {}", e)))?;

    let user_channel = format!("{room}:{user_id}");
    pubsub
      .subscribe(&user_channel)
      .await
      .map_err(|e| io::Error::other(format!("Failed to subscribe to user channel: {}", e)))?;

    let (tx, rx) = mpsc::unbounded_channel();

    let stream_handle = tokio::spawn(async move {
      {
        let mut stream = pubsub.on_message();

        loop {
          tokio::select! {
            _ = cancellation_token.cancelled() => {
              log::debug!("Redis pubsub stream cancelled for room: {}", room);
              break;
            }
            msg = stream.next() => {
              match msg {
                Some(msg) => {
                  match msg.get_payload::<String>() {
                    Ok(payload) => {
                      if tx.send(payload).is_err() {
                        break;
                      }
                    }
                    Err(e) => log::error!("Failed to get Redis message payload: {}", e),
                  }
                }
                None => {
                  break;
                }
              }
            }
          }
        }
      }

      if let Err(e) = pubsub.unsubscribe(&room).await {
        log::error!("Failed to unsubscribe from Redis room {}: {}", room, e);
      }
    });

    let stream = stream::unfold(rx, |mut rx| async {
      rx.recv().await.map(|payload| (payload, rx))
    });

    let guarded_stream = GuardedStream {
      stream: Box::pin(stream),
      handle: stream_handle,
    };

    Ok(Box::pin(guarded_stream))
  }
}
