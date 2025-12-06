use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};

use automerge::{ActorId, Automerge, sync::SyncDoc};
use axum::extract::ws::Message;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dashmap::{DashMap, mapref::entry::Entry};
use hashbrown::HashSet;
use once_cell::sync::Lazy;
use os_api::{HttpError, INTERNAL_ERROR};
use serde::Serialize;
use tokio::fs;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
  core::storage::{sled_storage_adapter::SledStorageAdapter, storage::Storage},
  router::ws::{
    constants::DATA_PATH_DOCUMENTS,
    entity::{
      DocumentUnavailableMessage, ErrorMessage, FromClientMessage, FromServerMessage, JoinMessage,
      PeerId, PeerMessage, PeerMetadata, RemoteHeadsChanged, RemoteSubscriptionControlMessage,
      RequestMessage, SyncMessage,
    },
  },
};

pub type PeerSender = mpsc::UnboundedSender<Message>;

struct StorageSystemInner {
  key: u64,
  storage_id: uuid::Uuid,
  storage: Arc<Storage<SledStorageAdapter>>,
  peer_senders: DashMap<PeerId, PeerSender>,
}

static SYNC_REGISTRY: Lazy<DashMap<u64, Arc<StorageSystemInner>>> = Lazy::new(DashMap::new);

#[derive(Clone)]
pub struct StorageSystem {
  inner: Arc<StorageSystemInner>,
}

impl StorageSystem {
  pub async fn get(base_path: &Path, aud: &str, sub: &str) -> Result<Self, HttpError> {
    let key = Self::storage_key(aud, sub);

    let inner = match SYNC_REGISTRY.entry(key) {
      Entry::Occupied(existing) => existing.get().clone(),
      Entry::Vacant(vacant) => {
        let storage_path = base_path
          .join(DATA_PATH_DOCUMENTS)
          .join(BASE64_URL_SAFE_NO_PAD.encode(aud.as_bytes()))
          .join(BASE64_URL_SAFE_NO_PAD.encode(sub.as_bytes()));

        if let Err(err) = fs::create_dir_all(&storage_path).await {
          log::error!(
            "failed to create document store storage path {}: {}",
            storage_path.display(),
            err
          );
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }

        let storage_adapter = match SledStorageAdapter::try_from(storage_path.as_path()) {
          Ok(adapter) => adapter,
          Err(err) => {
            log::error!(
              "failed to create document store storage adapter for path {}: {}",
              storage_path.display(),
              err
            );
            return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
          }
        };

        let storage = Arc::new(Storage::new(storage_adapter));
        let inner = Arc::new(StorageSystemInner {
          key,
          storage_id: uuid::Uuid::now_v7(),
          storage,
          peer_senders: DashMap::new(),
        });
        vacant.insert(inner.clone());
        inner
      }
    };

    Ok(Self { inner })
  }

  fn storage_key(aud: &str, sub: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    aud.hash(&mut hasher);
    sub.hash(&mut hasher);
    hasher.finish()
  }

  pub fn peer_id(&self) -> String {
    format!("server-{}", self.inner.key)
  }

  pub fn peer_metadata(&self) -> PeerMetadata {
    PeerMetadata {
      storage_id: self.inner.storage_id.to_string(),
      is_ephemeral: false,
    }
  }

  pub fn storage(&self) -> &Arc<Storage<SledStorageAdapter>> {
    &self.inner.storage
  }

  pub fn register_peer(&self, peer_id: PeerId, sender: PeerSender) {
    self.inner.peer_senders.insert(peer_id, sender);
  }

  pub fn unregister_peer(&self, peer_id: &str) {
    self.inner.peer_senders.remove(peer_id);
  }

  pub fn get_peer_ids(&self) -> HashSet<PeerId> {
    self
      .inner
      .peer_senders
      .iter()
      .map(|entry| entry.key().clone())
      .collect()
  }

  pub fn broadcast_to_peers(&self, sender_id: PeerId, mut from_server_message: FromServerMessage) {
    from_server_message.set_sender_id(self.peer_id());

    for entry in self.inner.peer_senders.iter() {
      let peer_id = entry.key();

      if peer_id == &sender_id {
        continue;
      }
      let mut message = from_server_message.clone();

      message.set_target_id(peer_id.clone());

      if let Some(bytes) = encode_to_bytes(&message) {
        if let Err(err) = entry.value().send(Message::Binary(bytes.into())) {
          log::error!("Failed to broadcast to peer {}: {}", peer_id, err);
        }
      }
    }
  }

  pub fn send_to_peer(&self, peer_id: PeerId, mut from_server_message: FromServerMessage) {
    from_server_message.set_sender_id(self.peer_id());
    from_server_message.set_target_id(peer_id.clone());

    if let Some(bytes) = encode_to_bytes(&from_server_message) {
      if let Some(entry) = self.inner.peer_senders.get(&peer_id) {
        if let Err(err) = entry.value().send(Message::Binary(bytes.into())) {
          log::error!("Failed to send to peer {}: {}", peer_id, err);
        }
      }
    }
  }

  fn handle_join(&self, join_message: JoinMessage, peer_sender: &PeerSender) -> Option<String> {
    let registered_peer_id = join_message.sender_id.clone();

    self.register_peer(join_message.sender_id.clone(), peer_sender.clone());

    let from_server_message = FromServerMessage::Peer(PeerMessage {
      sender_id: self.peer_id(),
      peer_metadata: self.peer_metadata(),
      target_id: join_message.sender_id,
    });

    if let Some(encoded) = encode_to_bytes(&from_server_message) {
      if let Err(err) = peer_sender.send(Message::Binary(encoded.into())) {
        log::error!("Failed to send peer message: {}", err);
      }
    }

    Some(registered_peer_id)
  }

  fn handle_sync(&self, sync_message: SyncMessage) {
    let document_id = match bs58_to_uuid(&sync_message.document_id) {
      Some(uuid) => uuid,
      None => {
        return;
      }
    };
    let message = match automerge::sync::Message::decode(&sync_message.data) {
      Ok(message) => message,
      Err(err) => {
        log::error!("Failed to decode sync message: {}", err);
        self.send_to_peer(
          sync_message.sender_id.clone(),
          FromServerMessage::Error(ErrorMessage {
            message: format!("Failed to decode sync message"),
            ..Default::default()
          }),
        );
        return;
      }
    };
    let mut document = match self.storage().load_document(document_id) {
      Ok(Some(document)) => document,
      Ok(None) => Automerge::new().with_actor(ActorId::from(document_id.as_bytes())),
      Err(err) => {
        log::error!("Failed to load document {}: {}", document_id, err);
        self.send_to_peer(
          sync_message.sender_id.clone(),
          FromServerMessage::Error(ErrorMessage {
            message: format!("Failed to load document"),
            ..Default::default()
          }),
        );
        return;
      }
    };

    let mut sync_state = automerge::sync::State::new();

    if let Err(err) = document.receive_sync_message(&mut sync_state, message) {
      log::error!("Failed to receive sync message: {}", err);
      self.send_to_peer(
        sync_message.sender_id.clone(),
        FromServerMessage::Error(ErrorMessage {
          message: format!("Failed to process sync message"),
          ..Default::default()
        }),
      );
      return;
    }
    let outgoing = match document.generate_sync_message(&mut sync_state) {
      Some(outgoing) => outgoing,
      None => {
        log::debug!("No sync message to send in response");
        return;
      }
    };

    match self.storage().save_document(document_id, &document) {
      Ok(_) => {}
      Err(err) => {
        log::error!("Failed to save document {}: {}", document_id, err);
        self.send_to_peer(
          sync_message.sender_id.clone(),
          FromServerMessage::Error(ErrorMessage {
            message: format!("Failed to save document"),
            ..Default::default()
          }),
        );
        return;
      }
    };

    self.broadcast_to_peers(
      sync_message.sender_id,
      FromServerMessage::Sync(SyncMessage {
        document_id: sync_message.document_id,
        data: outgoing.encode(),
        ..Default::default()
      }),
    );
  }

  fn handle_ephemeral(&self, ephemeral_message: crate::router::ws::entity::EphemeralMessage) {
    log::debug!("Received Ephemeral message: {:?}", ephemeral_message);
  }

  fn handle_request(&self, request_message: RequestMessage) {
    let document_id = match bs58_to_uuid(&request_message.document_id) {
      Some(uuid) => uuid,
      None => {
        return;
      }
    };

    let message = match automerge::sync::Message::decode(&request_message.data) {
      Ok(msg) => msg,
      Err(err) => {
        log::error!("Failed to decode sync message: {}", err);
        self.send_to_peer(
          request_message.sender_id.clone(),
          FromServerMessage::Error(ErrorMessage {
            message: format!("Failed to decode sync message"),
            ..Default::default()
          }),
        );
        return;
      }
    };

    let mut document = match self.storage().load_document(document_id) {
      Ok(Some(document)) => document,
      Ok(None) => {
        self.send_to_peer(
          request_message.sender_id.clone(),
          FromServerMessage::DocumentUnavailable(DocumentUnavailableMessage {
            document_id: request_message.document_id.clone(),
            ..Default::default()
          }),
        );
        return;
      }
      Err(err) => {
        log::error!("Failed to load document {}: {}", document_id, err);
        self.send_to_peer(
          request_message.sender_id.clone(),
          FromServerMessage::Error(ErrorMessage {
            message: format!("Failed to load document"),
            ..Default::default()
          }),
        );
        return;
      }
    };

    let mut sync_state = automerge::sync::State::new();

    if let Err(err) = document.receive_sync_message(&mut sync_state, message) {
      log::error!("Failed to receive sync message: {}", err);
      return;
    }

    let outgoing = match document.generate_sync_message(&mut sync_state) {
      Some(outgoing) => outgoing,
      None => {
        log::debug!("No sync message to send in response");
        return;
      }
    };

    let response = FromServerMessage::Sync(SyncMessage {
      document_id: request_message.document_id,
      data: outgoing.encode(),
      ..Default::default()
    });

    self.send_to_peer(request_message.sender_id, response);
  }

  fn handle_document_unavailable(&self, document_unavailable_message: DocumentUnavailableMessage) {
    log::debug!(
      "Received DocumentUnavailable message: {:?}",
      document_unavailable_message
    );
  }

  fn handle_remote_subscription_control(
    &self,
    remote_subscription_control_message: RemoteSubscriptionControlMessage,
  ) {
    log::debug!(
      "Received RemoteSubscriptionControl message: {:?}",
      remote_subscription_control_message
    );
  }

  fn handle_remote_heads_changed(&self, remote_heads_changed: RemoteHeadsChanged) {
    log::debug!(
      "Received RemoteHeadsChanged message: {:?}",
      remote_heads_changed
    );
  }

  pub async fn handle_ws_messages(
    &self,
    peer_sender: PeerSender,
    mut ws_receiver: impl futures::stream::StreamExt<
      Item = Result<axum::extract::ws::Message, axum::Error>,
    > + Unpin,
  ) -> Option<String> {
    let mut registered_peer_id: Option<String> = None;

    while let Some(result) = ws_receiver.next().await {
      let msg = match result {
        Ok(m) => m,
        Err(e) => {
          log::error!("WebSocket receive error: {}", e);
          continue;
        }
      };
      match msg {
        Message::Text(text) => {
          log::debug!("Received text: {}", text);
        }
        Message::Binary(bin) => {
          let client_msg: FromClientMessage = match ciborium::from_reader(bin.as_ref()) {
            Ok(msg) => msg,
            Err(err) => {
              log::error!("Failed to deserialize CBOR message: {}", err);
              continue;
            }
          };
          match client_msg {
            FromClientMessage::Join(join_message) => {
              if let Some(peer_id) = self.handle_join(join_message, &peer_sender) {
                registered_peer_id = Some(peer_id);
              }
            }
            FromClientMessage::Sync(sync_message) => {
              self.handle_sync(sync_message);
            }
            FromClientMessage::Ephemeral(ephemeral_message) => {
              self.handle_ephemeral(ephemeral_message);
            }
            FromClientMessage::Request(request_message) => {
              self.handle_request(request_message);
            }
            FromClientMessage::DocumentUnavailable(document_unavailable_message) => {
              self.handle_document_unavailable(document_unavailable_message);
            }
            FromClientMessage::RemoteSubscriptionControl(remote_subscription_control_message) => {
              self.handle_remote_subscription_control(remote_subscription_control_message);
            }
            FromClientMessage::RemoteHeadsChanged(remote_heads_changed) => {
              self.handle_remote_heads_changed(remote_heads_changed);
            }
          }
        }
        Message::Ping(p) => {
          log::debug!("Received ping: {:?}", p);
        }
        Message::Pong(p) => {
          log::debug!("Received pong: {:?}", p);
        }
        Message::Close(frame) => {
          log::debug!("Received close: {:?}", frame);
          break;
        }
      }
    }

    registered_peer_id
  }
}

impl Drop for StorageSystem {
  fn drop(&mut self) {
    if Arc::strong_count(&self.inner) == 2 {
      SYNC_REGISTRY.remove(&self.inner.key);
    }
  }
}

fn encode_to_bytes<T>(value: &T) -> Option<Vec<u8>>
where
  T: Serialize + ?Sized,
{
  let mut encoded = Vec::new();

  match ciborium::ser::into_writer(value, &mut encoded) {
    Ok(()) => Some(encoded),
    Err(err) => {
      log::error!("Failed to serialize: {}", err);
      None
    }
  }
}

fn bs58_to_uuid(bs58_string: &str) -> Option<Uuid> {
  let bytes = match bs58::decode(bs58_string).into_vec() {
    Ok(bytes) => bytes,
    Err(err) => {
      log::error!("Failed to decode base58 string: {}", err);
      return None;
    }
  };

  let uuid_bytes = match bytes.len() {
    16 => &bytes,
    20 => &bytes[4..20],
    _ => {
      log::error!(
        "Decoded bytes length is not 16 or 20: {} bytes",
        bytes.len()
      );
      return None;
    }
  };

  match Uuid::from_slice(uuid_bytes) {
    Ok(uuid) => Some(uuid),
    Err(err) => {
      log::error!("Failed to convert bytes to UUID: {}", err);
      None
    }
  }
}
