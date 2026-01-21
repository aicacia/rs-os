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
use once_cell::sync::Lazy;
use os_api::{HttpError, INTERNAL_ERROR};
use serde::Serialize;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
  core::storage::{fs_storage_adapter::FSStorageAdapter, storage::Storage},
  router::ws::{
    constants::DATA_PATH_DOCUMENTS,
    entity::{
      DocumentId, DocumentUnavailableMessage, ErrorMessage, FromClientMessage, FromServerMessage,
      JoinMessage, PeerId, PeerMessage, PeerMetadata, RemoteHeadsChanged,
      RemoteSubscriptionControlMessage, RequestMessage, SyncMessage,
    },
  },
};

pub type PeerSender = mpsc::UnboundedSender<Message>;

struct StorageSystemInner {
  key: u64,
  storage_id: uuid::Uuid,
  storage: Arc<Storage<FSStorageAdapter>>,
  peer_senders: DashMap<PeerId, PeerSender>,
  sync_states: DashMap<PeerId, DashMap<DocumentId, automerge::sync::State>>,
}

static SYNC_REGISTRY: Lazy<DashMap<u64, Arc<StorageSystemInner>>> = Lazy::new(DashMap::new);

#[derive(Clone)]
pub struct StorageSystem {
  inner: Arc<StorageSystemInner>,
}

impl StorageSystem {
  pub async fn get(base_path: &Path, unique_key: &str) -> Result<Self, HttpError> {
    let key = Self::storage_key(unique_key);

    let inner = match SYNC_REGISTRY.entry(key) {
      Entry::Occupied(existing) => existing.get().clone(),
      Entry::Vacant(vacant) => {
        let storage_path = base_path
          .join(DATA_PATH_DOCUMENTS)
          .join(BASE64_URL_SAFE_NO_PAD.encode(unique_key.as_bytes()));

        let storage_adapter = match FSStorageAdapter::try_from(storage_path.as_path()) {
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
          sync_states: DashMap::new(),
        });
        vacant.insert(inner.clone());
        inner
      }
    };

    Ok(Self { inner })
  }

  fn storage_key(unique_key: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    unique_key.hash(&mut hasher);
    hasher.finish()
  }

  fn peer_id(&self) -> String {
    format!("server-{}", self.inner.key)
  }

  fn peer_metadata(&self) -> PeerMetadata {
    PeerMetadata {
      storage_id: self.inner.storage_id.to_string(),
      is_ephemeral: false,
    }
  }

  fn storage(&self) -> &Storage<FSStorageAdapter> {
    &self.inner.storage
  }

  fn register_peer(&self, peer_id: PeerId, sender: PeerSender) {
    self.inner.peer_senders.insert(peer_id, sender);
  }

  fn unregister_peer(&self, peer_id: &PeerId) {
    self.inner.peer_senders.remove(peer_id);
    self.inner.sync_states.remove(peer_id);
  }

  fn drop_peer(&self, peer_id: &PeerId) {
    self.inner.peer_senders.remove(peer_id);
    self.inner.sync_states.remove(peer_id);
  }

  fn broadcast_to_peers(&self, sender_id: PeerId, mut from_server_message: FromServerMessage) {
    from_server_message.set_sender_id(self.peer_id());

    let peers_to_send: Vec<(PeerId, PeerSender)> = self
      .inner
      .peer_senders
      .iter()
      .filter_map(|entry| {
        if entry.key() == &sender_id {
          None
        } else {
          Some((entry.key().clone(), entry.value().clone()))
        }
      })
      .collect();

    let mut peers_to_drop = Vec::new();

    for (peer_id, sender) in peers_to_send {
      let mut message = from_server_message.clone();
      message.set_target_id(peer_id.clone());

      if let Some(bytes) = encode_to_bytes(&message) {
        if let Err(err) = sender.send(Message::Binary(bytes.into())) {
          log::error!("Failed to broadcast to peer {}: {}", peer_id, err);
          peers_to_drop.push(peer_id);
        }
      }
    }

    for peer_id in peers_to_drop {
      self.drop_peer(&peer_id);
    }
  }

  fn send_to_peer(&self, peer_id: PeerId, mut from_server_message: FromServerMessage) {
    from_server_message.set_sender_id(self.peer_id());
    from_server_message.set_target_id(peer_id.clone());

    log::debug!("Sending message to peer: {:?}", from_server_message);

    if let Some(bytes) = encode_to_bytes(&from_server_message) {
      if let Some(sender) = self
        .inner
        .peer_senders
        .get(&peer_id)
        .map(|e| e.value().clone())
      {
        if let Err(err) = sender.send(Message::Binary(bytes.into())) {
          log::error!("Failed to send to peer {}: {}", peer_id, err);
          self.drop_peer(&peer_id);
        }
      } else {
        log::warn!("No sender found for peer {}", peer_id);
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
        self.drop_peer(&registered_peer_id);
        return None;
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

    let mut document: Automerge = match self.storage().load_document(document_id) {
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

    let mut sync_state = {
      self
        .inner
        .sync_states
        .entry(sync_message.sender_id.clone())
        .or_insert_with(DashMap::new)
        .entry(sync_message.document_id.clone())
        .or_insert_with(automerge::sync::State::new)
        .clone()
    };

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

    let outgoing: automerge::sync::Message = match document.generate_sync_message(&mut sync_state) {
      Some(outgoing) => outgoing,
      None => {
        log::debug!("No sync message to send in response");
        {
          if let Some(peer_states) = self.inner.sync_states.get(&sync_message.sender_id) {
            peer_states.insert(sync_message.document_id.clone(), sync_state);
          }
        }
        return;
      }
    };

    {
      if let Some(peer_states) = self.inner.sync_states.get(&sync_message.sender_id) {
        peer_states.insert(sync_message.document_id.clone(), sync_state);
      }
    }

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

    let mut document: Automerge = match self.storage().load_document(document_id) {
      Ok(Some(document)) => document,
      Ok(None) => {
        log::debug!("Document not found: {}", request_message.document_id);
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

    let mut sync_state = self
      .inner
      .sync_states
      .entry(request_message.sender_id.clone())
      .or_insert_with(DashMap::new)
      .entry(request_message.document_id.clone())
      .or_insert_with(automerge::sync::State::new)
      .clone();

    if let Err(err) = document.receive_sync_message(&mut sync_state, message) {
      log::error!("Failed to receive sync message: {}", err);
      return;
    }

    let outgoing: automerge::sync::Message = match document.generate_sync_message(&mut sync_state) {
      Some(outgoing) => outgoing,
      None => {
        log::debug!("No sync message to send in response");
        if let Some(peer_states) = self.inner.sync_states.get(&request_message.sender_id) {
          peer_states.insert(request_message.document_id.clone(), sync_state);
        }
        return;
      }
    };

    if let Some(peer_states) = self.inner.sync_states.get(&request_message.sender_id) {
      peer_states.insert(request_message.document_id.clone(), sync_state);
    }

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

  pub fn close(&self) {
    let peer_ids: Vec<PeerId> = self
      .inner
      .peer_senders
      .iter()
      .map(|e| e.key().clone())
      .collect();
    for peer_id in peer_ids {
      self.unregister_peer(&peer_id);
    }

    if let Err(err) = self.storage().flush() {
      log::error!("Failed to flush storage on close: {}", err);
    }
  }

  pub async fn handle_ws_messages(
    &self,
    peer_sender: PeerSender,
    mut ws_receiver: impl futures::stream::StreamExt<
      Item = Result<axum::extract::ws::Message, axum::Error>,
    > + Unpin,
  ) {
    let mut registered_peer_id: Option<String> = None;

    while let Some(result) = ws_receiver.next().await {
      let msg = match result {
        Ok(m) => m,
        Err(e) => {
          log::debug!("WebSocket receive error, closing stream: {}", e);
          break;
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
          log::debug!("Received binary message: {:?}", client_msg);
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

    if let Some(peer_id) = registered_peer_id {
      self.unregister_peer(&peer_id);
    }
  }
}

impl Drop for StorageSystem {
  fn drop(&mut self) {
    self.close();

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
