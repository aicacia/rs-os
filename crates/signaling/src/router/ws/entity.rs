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
  Join {
    from: String,
  },
  Peers {
    user_id: String,
    peers: Vec<String>,
  },
  Message {
    from: String,
    payload: serde_json::Value,
  },
  Leave {
    from: String,
  },
}
