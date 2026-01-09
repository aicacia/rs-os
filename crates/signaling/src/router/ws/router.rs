use std::sync::Arc;
use std::time::Duration;

use axum::{
  extract::{
    Query, State,
    ws::{Message, WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use os_api::{BasicClaims, HttpError, parse_token_data};
use tokio_util::sync::CancellationToken;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  ws::{
    constants::TAG,
    entity::{ClientMessage, ServerMessage, WSAuthorizationRequest, WSRoomRequest},
    pubsub::PubSub,
  },
};

const DEFAULT_BUFFER_CAPACITY: usize = 64;
const SEND_TIMEOUT_SECS: u64 = 10;
const CLEANUP_TIMEOUT_SECS: u64 = 5;

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

  let user_id = uuid::Uuid::now_v7().to_string();
  let room = format!("user:{}", claims.user);

  if state.cancellation_token.is_cancelled() {
    log::info!("WebSocket connection attempt during server shutdown");
    return HttpError::internal_error()
      .with_application_error("Server is shutting down")
      .into_response();
  }

  ws.on_upgrade(move |socket| async move {
    match handle_ws(
      socket,
      user_id.clone(),
      room.clone(),
      state.pubsub,
      state.cancellation_token,
    )
    .await
    {
      Ok(()) => {
        log::debug!(
          "Client WebSocket connection for user {} in room {} closed",
          user_id,
          room
        );
      }
      Err(err) => {
        log::error!("WebSocket error: {}", err);
      }
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

  let user_id = uuid::Uuid::now_v7().to_string();
  let room = format!("client:{}", claims.client);

  if state.cancellation_token.is_cancelled() {
    log::info!("WebSocket connection attempt during server shutdown");
    return HttpError::internal_error()
      .with_application_error("Server is shutting down")
      .into_response();
  }

  ws.on_upgrade(move |socket| async move {
    match handle_ws(
      socket,
      user_id.clone(),
      room.clone(),
      state.pubsub,
      state.cancellation_token,
    )
    .await
    {
      Ok(()) => {
        log::debug!(
          "Client WebSocket connection for user {} in room {} closed",
          user_id,
          room
        );
      }
      Err(err) => {
        log::error!("WebSocket error: {}", err);
      }
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
  let user_id = uuid::Uuid::now_v7().to_string();
  let room = format!("anonymous:{}", room_request.room);

  if state.cancellation_token.is_cancelled() {
    log::info!("WebSocket connection attempt during server shutdown");
    return HttpError::internal_error()
      .with_application_error("Server is shutting down")
      .into_response();
  }

  ws.on_upgrade(move |socket| async move {
    match handle_ws(
      socket,
      user_id.clone(),
      room.clone(),
      state.pubsub,
      state.cancellation_token,
    )
    .await
    {
      Ok(()) => {
        log::debug!(
          "Anonymous WebSocket connection for user {} in room {} closed",
          user_id,
          room
        );
      }
      Err(err) => {
        log::error!("WebSocket error: {}", err);
      }
    }
  })
  .into_response()
}

async fn handle_ws(
  socket: WebSocket,
  user_id: String,
  room: String,
  pubsub: Arc<PubSub>,
  cancellation_token: CancellationToken,
) -> Result<(), HttpError> {
  let result = handle_ws_inner(
    socket,
    user_id.clone(),
    room.clone(),
    pubsub.clone(),
    cancellation_token,
  )
  .await;

  log::debug!(
    "Starting WebSocket cleanup for user {} in room {}",
    user_id,
    room
  );

  let cleanup_timeout = Duration::from_secs(CLEANUP_TIMEOUT_SECS);
  let cleanup = async {
    if let Err(e) = pubsub.remove_user(&room, &user_id).await {
      log::error!(
        "Failed to remove user {} from room {} during cleanup: {}",
        user_id,
        room,
        e
      );
    }

    let leave_msg = ServerMessage::Leave {
      from: user_id.clone(),
    };
    if let Ok(leave_json) = serde_json::to_string(&leave_msg)
      && let Err(e) = pubsub.broadcast(&room, &leave_json).await
    {
      log::error!(
        "Failed to publish leave message for room {} during cleanup: {}",
        room,
        e
      );
    }
  };

  match tokio::time::timeout(cleanup_timeout, cleanup).await {
    Ok(_) => {
      log::debug!(
        "WebSocket connection for user {} in room {} cleaned up",
        user_id,
        room
      );
    }
    Err(_) => {
      log::warn!(
        "WebSocket cleanup timed out after {:?} for room {}",
        cleanup_timeout,
        room
      );
    }
  }

  result
}

async fn handle_ws_inner(
  socket: WebSocket,
  user_id: String,
  room: String,
  pubsub: Arc<PubSub>,
  cancellation_token: CancellationToken,
) -> Result<(), HttpError> {
  let peers = pubsub.get_peers(&room).await.map_err(|e| {
    log::error!("Failed to get peers: {}", e);
    HttpError::internal_error()
  })?;

  pubsub.add_user(&room, &user_id).await.map_err(|e| {
    log::error!("Failed to add user to room: {}", e);
    HttpError::internal_error()
  })?;

  pubsub
    .broadcast(
      &room,
      &serde_json::to_string(&ServerMessage::Join {
        from: user_id.clone(),
      })
      .map_err(|e| {
        log::error!("Failed to serialize join message: {}", e);
        HttpError::internal_error()
      })?,
    )
    .await
    .map_err(|e| {
      log::error!("Failed to publish join message: {}", e);
      HttpError::internal_error()
    })?;

  let (mut ws_sender, mut ws_receiver) = socket.split();

  if let Err(e) = ws_sender
    .send(Message::text(
      serde_json::to_string(&ServerMessage::Welcome {
        id: user_id.clone(),
        peers,
      })
      .map_err(|e| {
        log::error!("Failed to serialize peers message: {}", e);
        HttpError::internal_error()
      })?,
    ))
    .await
  {
    log::error!("Failed to send peers message: {}", e);
    return Err(HttpError::internal_error());
  }

  let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(DEFAULT_BUFFER_CAPACITY);

  let mut pubsub_stream = pubsub
    .subscribe(&room, &user_id, cancellation_token.clone())
    .await
    .map_err(|e| {
      log::error!("Failed to subscribe to room: {}", e);
      HttpError::internal_error()
    })?;

  let pubsub_user_id = user_id.clone();
  let pubsub_tx = tx.clone();
  let pubsub_task = tokio::spawn({
    let cancellation_token = cancellation_token.clone();
    async move {
      loop {
        tokio::select! {
          _ = cancellation_token.cancelled() => {
            log::debug!("PubSub task cancelled");
            break;
          }
          payload = pubsub_stream.next() => {
            let Some(payload) = payload else {
              break;
            };

            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&payload) {
              match server_msg {
                ServerMessage::Join { from: msg_user_id }
                | ServerMessage::Leave { from: msg_user_id } => {
                  if msg_user_id == pubsub_user_id {
                    continue;
                  }
                }
                ServerMessage::Welcome { .. } => {
                  continue;
                }
                ServerMessage::Message { .. } => {}
              }
            }

            if pubsub_tx.send(Message::text(payload)).await.is_err() {
              log::debug!("Message channel closed, pubsub task exiting");
              break;
            }
          }
        }
      }
    }
  });

  let ws_sender_task = tokio::spawn({
    let cancellation_token = cancellation_token.clone();
    async move {
      loop {
        let msg = tokio::select! {
          _ = cancellation_token.cancelled() => {
            log::debug!("WebSocket sender task cancelled");
            break;
          }
          msg = rx.recv() => {
            match msg {
              Some(msg) => msg,
              None => break,
            }
          }
        };

        match tokio::time::timeout(Duration::from_secs(SEND_TIMEOUT_SECS), ws_sender.send(msg))
          .await
        {
          Ok(Ok(())) => {}
          Ok(Err(e)) => {
            log::error!("WebSocket send error: {}", e);
            break;
          }
          Err(_) => {
            log::warn!("WebSocket send timeout, closing connection due to slow client");
            break;
          }
        }
      }
    }
  });

  let tx_control = tx.clone();

  loop {
    tokio::select! {
      _ = cancellation_token.cancelled() => {
        log::debug!("WebSocket receiver loop cancelled, closing connection");
        break;
      }
      msg = ws_receiver.next() => {
        let Some(msg) = msg else {
          break;
        };

        match msg {
          Ok(Message::Text(text)) => {
            let client_msg = match serde_json::from_str::<ClientMessage>(&text) {
              Ok(msg) => msg,
              Err(e) => {
                log::error!("Failed to parse client message: {}", e);
                continue;
              }
            };

            match client_msg {
              ClientMessage::Send { to, payload } => {
                let server_msg = ServerMessage::Message {
                  from: user_id.clone(),
                  payload,
                };
                let msg_json = match serde_json::to_string(&server_msg) {
                  Ok(json) => json,
                  Err(e) => {
                    log::error!("Failed to serialize message: {}", e);
                    continue;
                  }
                };

                if let Err(e) = pubsub.send(&room, &to, &msg_json).await {
                  log::error!("Failed to send message to user {}: {}", to, e);
                }
              }
              ClientMessage::Broadcast { payload } => {
                let server_msg = ServerMessage::Message {
                  from: user_id.clone(),
                  payload,
                };
                let msg_json = match serde_json::to_string(&server_msg) {
                  Ok(json) => json,
                  Err(e) => {
                    log::error!("Failed to serialize message: {}", e);
                    continue;
                  }
                };

                if let Err(e) = pubsub.broadcast(&room, &msg_json).await {
                  log::error!("Failed to publish broadcast message: {}", e);
                }
              }
            }
          }
          Ok(Message::Ping(payload)) => {
            if tx_control.send(Message::Pong(payload)).await.is_err() {
              log::debug!("WebSocket sender channel closed, stopping ping handling");
              break;
            }
          }
          Ok(Message::Pong(_)) => {
          }
          Ok(Message::Close(frame)) => {
            log::debug!("WebSocket close frame received, closing connection");
            let _ = tx_control.send(Message::Close(frame)).await;
            break;
          }
          Err(e) => {
            log::error!("WebSocket error: {}", e);
            break;
          }
          _ => {
            log::warn!("Unsupported WebSocket message received");
          }
        }
      }
    }
  }

  pubsub_task.abort();
  ws_sender_task.abort();

  if let Err(join_err) = pubsub_task.await {
    log::warn!("PubSub task join error: {}", join_err);
  }
  if let Err(join_err) = ws_sender_task.await {
    log::warn!("WebSocket sender task join error: {}", join_err);
  }

  Ok(())
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(user))
    .routes(routes!(client))
    .routes(routes!(anonymous))
    .with_state(state)
}
