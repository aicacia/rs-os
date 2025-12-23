use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct WSAuthorizationRequest {
  pub token: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct WSRoomRequest {
  pub room: String,
}
