use axum::{
  extract::{
    Query, State,
    ws::{WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use os_api::{BasicClaims, HttpError, parse_token_data};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  ws::{
    constants::TAG,
    entity::{WSAuthorizationRequest, WSRoomRequest},
  },
};

#[utoipa::path(
  get,
  path = "/user",
  tags = [TAG],
  params(WSAuthorizationRequest),
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn user(
  State(_state): State<RouterState>,
  Query(authorization_request): Query<WSAuthorizationRequest>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  let claims = match parse_token_data::<BasicClaims>(&authorization_request.token).await {
    Ok(token_data) => token_data.claims,
    Err(err) => {
      log::warn!("unauthorized WebSocket connection attempt: {}", err);
      return HttpError::unauthorized()
        .with_application_error("Unauthorized")
        .into_response();
    }
  };

  let user_id = claims.user;

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_user_ws(socket, user_id).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

async fn handle_user_ws(_socket: WebSocket, _user_id: i64) -> Result<(), HttpError> {
  Ok(())
}

#[utoipa::path(
  get,
  path = "/client",
  tags = [TAG],
  params(WSAuthorizationRequest),
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn client(
  State(_state): State<RouterState>,
  Query(authorization_request): Query<WSAuthorizationRequest>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  let claims = match parse_token_data::<BasicClaims>(&authorization_request.token).await {
    Ok(token_data) => token_data.claims,
    Err(err) => {
      log::warn!("unauthorized WebSocket connection attempt: {}", err);
      return HttpError::unauthorized()
        .with_application_error("Unauthorized")
        .into_response();
    }
  };

  let user_id = claims.user;
  let client_id = claims.client;

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_client_ws(socket, user_id, client_id).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

async fn handle_client_ws(
  _socket: WebSocket,
  _user: i64,
  _client: String,
) -> Result<(), HttpError> {
  Ok(())
}

#[utoipa::path(
  get,
  path = "/room",
  tags = [TAG],
  params(WSRoomRequest),
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn room(
  State(_state): State<RouterState>,
  Query(room_request): Query<WSRoomRequest>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  let room = room_request.room;

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_room_ws(socket, room).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

async fn handle_room_ws(_socket: WebSocket, _room: String) -> Result<(), HttpError> {
  Ok(())
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(user))
    .routes(routes!(client))
    .routes(routes!(room))
    .with_state(state)
}
