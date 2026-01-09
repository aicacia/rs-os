use std::{collections::HashSet, io};

use dashmap::DashMap;
use futures_util::stream;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use super::{MessageStream, pubsub::PubSubAdapterInternal};

#[derive(Default)]
pub struct InMemoryPubSub {
  rooms: DashMap<String, InMemoryRoom>,
  broadcast_capacity: usize,
}

#[derive(Clone)]
struct InMemoryRoom {
  users: HashSet<String>,
  sender: broadcast::Sender<String>,
}

impl InMemoryPubSub {
  pub fn new() -> Self {
    Self::with_capacity(64)
  }

  pub fn with_capacity(broadcast_capacity: usize) -> Self {
    Self {
      rooms: DashMap::new(),
      broadcast_capacity,
    }
  }

  fn ensure_room(&self, room: &str) -> broadcast::Sender<String> {
    self
      .rooms
      .entry(room.to_owned())
      .or_insert_with(|| {
        let (sender, _) = broadcast::channel(self.broadcast_capacity);
        InMemoryRoom {
          users: HashSet::new(),
          sender,
        }
      })
      .sender
      .clone()
  }
}

#[async_trait::async_trait]
impl PubSubAdapterInternal for InMemoryPubSub {
  async fn get_peers(&self, room: &str) -> io::Result<Vec<String>> {
    let room = room.to_owned();
    let peers = self
      .rooms
      .get(&room)
      .map(|entry| entry.users.iter().cloned().collect())
      .unwrap_or_default();
    Ok(peers)
  }

  async fn add_user(&self, room: &str, user_id: &str) -> io::Result<()> {
    let room = room.to_owned();
    let user_id = user_id.to_owned();

    let mut entry = self.rooms.entry(room).or_insert_with(|| {
      let (sender, _) = broadcast::channel(self.broadcast_capacity);
      InMemoryRoom {
        users: HashSet::new(),
        sender,
      }
    });
    entry.users.insert(user_id);
    Ok(())
  }

  async fn remove_user(&self, room: &str, user_id: &str) -> io::Result<()> {
    let room = room.to_owned();
    let user_id = user_id.to_owned();

    let should_cleanup = if let Some(mut entry) = self.rooms.get_mut(&room) {
      entry.users.remove(&user_id);
      entry.users.is_empty()
    } else {
      false
    };

    if should_cleanup {
      self.rooms.remove(&room);
    }

    Ok(())
  }

  async fn broadcast(&self, room: &str, payload: &str) -> io::Result<()> {
    let room = room.to_owned();
    let payload = payload.to_owned();

    let sender = self.ensure_room(&room);
    // If there are no receivers, broadcast::Sender::send returns an error.
    // Treat that as a no-op rather than a failure.
    let _ = sender.send(payload);
    Ok(())
  }

  async fn send(&self, room: &str, user_id: &str, payload: &str) -> io::Result<()> {
    let room = room.to_owned();
    let user_id = user_id.to_owned();
    let payload = payload.to_owned();

    if let Some(room_data) = self.rooms.get(&room) {
      if room_data.users.contains(&user_id) {
        // If there are no receivers (e.g., client disconnected), treat as no-op.
        let _ = room_data.sender.send(payload);
      }
    }
    Ok(())
  }

  async fn subscribe(
    &self,
    room: &str,
    user_id: &str,
    cancellation_token: CancellationToken,
  ) -> io::Result<MessageStream> {
    let room = room.to_owned();
    let user_id = user_id.to_owned();

    let receiver = self.ensure_room(&room).subscribe();

    let stream = stream::unfold(
      (receiver, room.clone(), user_id, cancellation_token),
      |(mut rx, room, user_id, token)| async move {
        loop {
          tokio::select! {
            _ = token.cancelled() => {
              log::debug!("In-memory pubsub stream cancelled for room: {}", room);
              return None;
            }
            result = rx.recv() => {
              match result {
                Ok(payload) => return Some((payload, (rx, room, user_id, token))),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                  log::warn!("In-memory pubsub lagged by {skipped} messages for room {room}");
                  continue;
                }
                Err(_) => return None,
              }
            }
          }
        }
      },
    );

    let stream: MessageStream = Box::pin(stream);

    Ok(stream)
  }
}
