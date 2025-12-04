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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub data: Vec<u8>,
  pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EphemeralMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub count: u64,
  pub session_id: SessionId,
  pub document_id: DocumentId,
  pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUnavailableMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub data: Vec<u8>,
  pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSubscriptionControlMessage {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub add: Option<Vec<StorageId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub remove: Option<Vec<StorageId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteHeadsChanged {
  pub sender_id: PeerId,
  pub target_id: PeerId,
  pub document_id: DocumentId,
  pub new_heads: std::collections::HashMap<StorageId, HeadsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadsInfo {
  pub heads: Vec<String>,
  pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerMetadata {
  pub storage_id: String,
  pub is_ephemeral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMessage {
  pub sender_id: PeerId,
  pub peer_metadata: PeerMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerMessage {
  pub sender_id: PeerId,
  pub peer_metadata: PeerMetadata,
  pub target_id: PeerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
