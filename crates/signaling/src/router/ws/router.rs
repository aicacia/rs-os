use axum::{
  extract::{
    Query, State,
    ws::{Message, WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use os_api::{BasicClaims, HttpError, parse_token_data};
use redis::AsyncCommands;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  ws::{
    constants::TAG,
    entity::{RoomMessage, WSAuthorizationRequest, WSRoomRequest},
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
  State(state): State<RouterState>,
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

  let user_id = claims.sub;
  let room = format!("user:{}", claims.user);

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_ws(socket, user_id, room, state).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
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
  State(state): State<RouterState>,
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

  let user_id = claims.sub;
  let room = format!("client:{}", claims.client);

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_ws(socket, user_id, room, state).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

#[utoipa::path(
  get,
  path = "/anonymous",
  tags = [TAG],
  params(WSRoomRequest),
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn anonymous(
  State(state): State<RouterState>,
  Query(room_request): Query<WSRoomRequest>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  let user_id = uuid::Uuid::new_v4().to_string();
  let room = format!("anonymous:{}", room_request.room);

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_ws(socket, user_id, room, state).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

async fn handle_ws(
  socket: WebSocket,
  user_id: String,
  room: String,
  state: RouterState,
) -> Result<(), HttpError> {
  let room_channel = room.clone();
  let room_users_key = format!("{}:users", room);

  let mut pub_conn = state
    .redis_client
    .get_multiplexed_async_connection()
    .await
    .map_err(|e| {
      log::error!("Failed to get Redis pub connection: {}", e);
      HttpError::internal_error()
    })?;

  let mut pubsub = state.redis_client.get_async_pubsub().await.map_err(|e| {
    log::error!("Failed to get Redis pubsub connection: {}", e);
    HttpError::internal_error()
  })?;

  pubsub.subscribe(&room_channel).await.map_err(|e| {
    log::error!("Failed to subscribe to room channel: {}", e);
    HttpError::internal_error()
  })?;

  let peers: Vec<String> = pub_conn.smembers(&room_users_key).await.map_err(|e| {
    log::error!("Failed to get room users: {}", e);
    HttpError::internal_error()
  })?;

  pub_conn
    .sadd::<_, _, ()>(&room_users_key, &user_id)
    .await
    .map_err(|e| {
      log::error!("Failed to add user to room: {}", e);
      HttpError::internal_error()
    })?;

  let join_msg = RoomMessage::Join {
    user_id: user_id.clone(),
  };
  let join_json = serde_json::to_string(&join_msg).map_err(|e| {
    log::error!("Failed to serialize join message: {}", e);
    HttpError::internal_error()
  })?;

  pub_conn
    .publish::<_, _, ()>(&room_channel, &join_json)
    .await
    .map_err(|e| {
      log::error!("Failed to publish join message: {}", e);
      HttpError::internal_error()
    })?;

  let (mut ws_sender, mut ws_receiver) = socket.split();
  let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

  let peers_msg = RoomMessage::Peers { user_ids: peers };
  let peers_json = serde_json::to_string(&peers_msg).map_err(|e| {
    log::error!("Failed to serialize peers message: {}", e);
    HttpError::internal_error()
  })?;

  if let Err(e) = ws_sender.send(Message::text(peers_json)).await {
    log::error!("Failed to send peers message: {}", e);
    return Err(HttpError::internal_error());
  }

  let user_id_clone = user_id.clone();
  let redis_task = tokio::spawn(async move {
    let mut pubsub_stream = pubsub.on_message();
    while let Some(msg) = pubsub_stream.next().await {
      let payload: String = match msg.get_payload() {
        Ok(p) => p,
        Err(e) => {
          log::error!("Failed to get Redis message payload: {}", e);
          continue;
        }
      };
      if let Ok(room_msg) = serde_json::from_str::<RoomMessage>(&payload) {
        match room_msg {
          RoomMessage::Join {
            user_id: msg_user_id,
          }
          | RoomMessage::Leave {
            user_id: msg_user_id,
          } => {
            if msg_user_id == user_id_clone {
              continue;
            }
          }
          RoomMessage::Peers { .. } => {
            continue;
          }
          RoomMessage::Message { .. } => {}
        }
      }

      if let Err(e) = tx.send(payload) {
        log::error!("Failed to send message to channel: {}", e);
        break;
      }
    }
  });

  let ws_sender_task = tokio::spawn(async move {
    while let Some(payload) = rx.recv().await {
      if ws_sender.send(Message::text(payload)).await.is_err() {
        break;
      }
    }
  });

  while let Some(msg) = ws_receiver.next().await {
    match msg {
      Ok(Message::Text(text)) => {
        let room_msg = RoomMessage::Message {
          user_id: user_id.clone(),
          content: text.to_string(),
        };

        let msg_json = serde_json::to_string(&room_msg).map_err(|e| {
          log::error!("Failed to serialize message: {}", e);
          HttpError::internal_error()
        })?;

        pub_conn
          .publish::<_, _, ()>(&room_channel, &msg_json)
          .await
          .map_err(|e| {
            log::error!("Failed to publish message: {}", e);
            HttpError::internal_error()
          })?;
      }
      Ok(Message::Close(_)) => {
        break;
      }
      Err(e) => {
        log::error!("WebSocket error: {}", e);
        break;
      }
      _ => {}
    }
  }

  pub_conn
    .srem::<_, _, ()>(&room_users_key, &user_id)
    .await
    .map_err(|e| {
      log::error!("Failed to remove user from room: {}", e);
      HttpError::internal_error()
    })?;

  let leave_msg = RoomMessage::Leave {
    user_id: user_id.clone(),
  };
  let leave_json = serde_json::to_string(&leave_msg).map_err(|e| {
    log::error!("Failed to serialize leave message: {}", e);
    HttpError::internal_error()
  })?;

  pub_conn
    .publish::<_, _, ()>(&room_channel, &leave_json)
    .await
    .map_err(|e| {
      log::error!("Failed to publish leave message: {}", e);
      HttpError::internal_error()
    })?;

  redis_task.abort();
  ws_sender_task.abort();

  Ok(())
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(user))
    .routes(routes!(client))
    .routes(routes!(anonymous))
    .with_state(state)
}
