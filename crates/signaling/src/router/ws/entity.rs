use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct WSAuthorizationRequest {
  pub token: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct WSRoomRequest {
  pub room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoomMessage {
  Join { user_id: String },
  Peers { user_ids: Vec<String> },
  Message { user_id: String, content: String },
  Leave { user_id: String },
}
