use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct WSAuthorizationRequest {
  pub token: String,
}

pub type PeerId = String;
pub type DocumentId = String;
pub type StorageId = String;
pub type SessionId = String;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub document_id: DocumentId,
  #[serde(with = "serde_bytes")]
  pub data: Vec<u8>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EphemeralMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub count: u64,
  pub session_id: SessionId,
  pub document_id: DocumentId,
  #[serde(with = "serde_bytes")]
  pub data: Vec<u8>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUnavailableMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub document_id: DocumentId,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub document_id: DocumentId,
  #[serde(with = "serde_bytes")]
  pub data: Vec<u8>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSubscriptionControlMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub add: Option<Vec<StorageId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub remove: Option<Vec<StorageId>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteHeadsChanged {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub document_id: DocumentId,
  pub new_heads: std::collections::HashMap<StorageId, HeadsInfo>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadsInfo {
  pub heads: Vec<String>,
  pub timestamp: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerMetadata {
  pub storage_id: String,
  pub is_ephemeral: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMessage {
  pub sender_id: PeerId,
  pub peer_metadata: PeerMetadata,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerMessage {
  pub sender_id: PeerId,
  pub peer_metadata: PeerMetadata,
  pub target_id: PeerId,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
  pub sender_id: PeerId,
  pub message: String,
  pub target_id: PeerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FromClientMessage {
  #[serde(rename = "join")]
  Join(JoinMessage),
  #[serde(rename = "sync")]
  Sync(SyncMessage),
  #[serde(rename = "ephemeral")]
  Ephemeral(EphemeralMessage),
  #[serde(rename = "request")]
  Request(RequestMessage),
  #[serde(rename = "doc-unavailable")]
  DocumentUnavailable(DocumentUnavailableMessage),
  #[serde(rename = "remote-subscription-change")]
  RemoteSubscriptionControl(RemoteSubscriptionControlMessage),
  #[serde(rename = "remote-heads-changed")]
  RemoteHeadsChanged(RemoteHeadsChanged),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FromServerMessage {
  #[serde(rename = "peer")]
  Peer(PeerMessage),
  #[serde(rename = "error")]
  Error(ErrorMessage),
  #[serde(rename = "sync")]
  Sync(SyncMessage),
  #[serde(rename = "ephemeral")]
  Ephemeral(EphemeralMessage),
  #[serde(rename = "request")]
  Request(RequestMessage),
  #[serde(rename = "doc-unavailable")]
  DocumentUnavailable(DocumentUnavailableMessage),
  #[serde(rename = "remote-subscription-change")]
  RemoteSubscriptionControl(RemoteSubscriptionControlMessage),
  #[serde(rename = "remote-heads-changed")]
  RemoteHeadsChanged(RemoteHeadsChanged),
}

impl FromServerMessage {
  pub fn set_sender_id(&mut self, sender_id: PeerId) {
    match self {
      FromServerMessage::Peer(msg) => msg.sender_id = sender_id,
      FromServerMessage::Error(msg) => msg.sender_id = sender_id,
      FromServerMessage::Sync(msg) => msg.sender_id = sender_id,
      FromServerMessage::Ephemeral(msg) => msg.sender_id = sender_id,
      FromServerMessage::Request(msg) => msg.sender_id = sender_id,
      FromServerMessage::DocumentUnavailable(msg) => msg.sender_id = sender_id,
      FromServerMessage::RemoteSubscriptionControl(msg) => msg.sender_id = sender_id,
      FromServerMessage::RemoteHeadsChanged(msg) => msg.sender_id = sender_id,
    }
  }

  pub fn set_target_id(&mut self, target_id: PeerId) {
    match self {
      FromServerMessage::Peer(msg) => msg.target_id = target_id,
      FromServerMessage::Error(msg) => msg.target_id = target_id,
      FromServerMessage::Sync(msg) => msg.target_id = target_id,
      FromServerMessage::Ephemeral(msg) => msg.target_id = target_id,
      FromServerMessage::Request(msg) => msg.target_id = target_id,
      FromServerMessage::DocumentUnavailable(msg) => msg.target_id = target_id,
      FromServerMessage::RemoteSubscriptionControl(msg) => msg.target_id = target_id,
      FromServerMessage::RemoteHeadsChanged(msg) => msg.target_id = target_id,
    }
  }
}
