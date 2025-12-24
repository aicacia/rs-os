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
use scopeguard::defer;
use tokio_util::sync::CancellationToken;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  ws::{
    constants::TAG,
    entity::{RoomMessage, WSAuthorizationRequest, WSRoomRequest},
    pubsub::PubSub,
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
    if let Err(err) = handle_ws(socket, user_id, room, state.pubsub).await {
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
    if let Err(err) = handle_ws(socket, user_id, room, state.pubsub).await {
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
    if let Err(err) = handle_ws(socket, user_id, room, state.pubsub).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

async fn handle_ws(
  socket: WebSocket,
  user_id: String,
  room: String,
  pubsub: Arc<PubSub>,
) -> Result<(), HttpError> {
  let cancellation_token = CancellationToken::new();

  let result = handle_ws_inner(
    socket,
    user_id.clone(),
    room.clone(),
    pubsub.clone(),
    cancellation_token.clone(),
  )
  .await;

  result
}

async fn handle_ws_inner(
  socket: WebSocket,
  user_id: String,
  room: String,
  pubsub: Arc<PubSub>,
  cancellation_token: CancellationToken,
) -> Result<(), HttpError> {
  // Add user to room
  pubsub.add_user(&room, &user_id).await.map_err(|e| {
    log::error!("Failed to add user to room: {}", e);
    HttpError::internal_error()
  })?;

  // Set up cleanup to run on scope exit
  let pubsub_cleanup = pubsub.clone();
  let room_cleanup = room.clone();
  let user_id_cleanup = user_id.clone();
  defer! {
    let pubsub = pubsub_cleanup;
    let room = room_cleanup;
    let user_id = user_id_cleanup;

    tokio::spawn(async move {
      if let Err(e) = pubsub.remove_user(&room, &user_id).await {
        log::error!(
          "Failed to remove user {} from room {} during cleanup: {}",
          user_id,
          room,
          e
        );
      }

      let leave_msg = RoomMessage::Leave { user_id };
      if let Ok(leave_json) = serde_json::to_string(&leave_msg) {
        if let Err(e) = pubsub.publish(&room, &leave_json).await {
          log::error!(
            "Failed to publish leave message for room {} during cleanup: {}",
            room,
            e
          );
        }
      }
    });
  }

  let peers = pubsub.get_peers(&room).await.map_err(|e| {
    log::error!("Failed to get peers: {}", e);
    HttpError::internal_error()
  })?;

  pubsub
    .publish(
      &room,
      &serde_json::to_string(&RoomMessage::Join {
        user_id: user_id.clone(),
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
  let (tx, mut rx) = tokio::sync::mpsc::channel(32);

  if let Err(e) = ws_sender
    .send(Message::text(
      serde_json::to_string(&RoomMessage::Peers { user_ids: peers }).map_err(|e| {
        log::error!("Failed to serialize peers message: {}", e);
        HttpError::internal_error()
      })?,
    ))
    .await
  {
    log::error!("Failed to send peers message: {}", e);
    return Err(HttpError::internal_error());
  }

  let pubsub_user_id = user_id.clone();
  let mut pubsub_stream = pubsub
    .subscribe(&room, cancellation_token.clone())
    .await
    .map_err(|e| {
      log::error!("Failed to subscribe to room: {}", e);
      HttpError::internal_error()
    })?;

  let mut pubsub_task_handle = tokio::spawn({
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

            if let Ok(room_msg) = serde_json::from_str::<RoomMessage>(&payload) {
              match room_msg {
                RoomMessage::Join {
                  user_id: msg_user_id,
                }
                | RoomMessage::Leave {
                  user_id: msg_user_id,
                } => {
                  if msg_user_id == pubsub_user_id {
                    continue;
                  }
                }
                RoomMessage::Peers { .. } => {
                  continue;
                }
                RoomMessage::Message { .. } => {}
              }
            }

            match tx.send(payload).await {
              Ok(_) => {}
              Err(tokio::sync::mpsc::error::SendError(_)) => {
                log::debug!("Message channel closed, pubsub task exiting");
                break;
              }
            }
          }
        }
      }
    }
  });

  let mut ws_sender_task_handle = tokio::spawn({
    let cancellation_token = cancellation_token.clone();
    async move {
      loop {
        tokio::select! {
          _ = cancellation_token.cancelled() => {
            log::debug!("WebSocket sender task cancelled");
            break;
          }
          payload = rx.recv() => {
            let Some(payload) = payload else {
              break;
            };

            match tokio::time::timeout(
              Duration::from_secs(10),
              ws_sender.send(Message::text(payload)),
            )
            .await
            {
              Ok(Ok(())) => {
              }
              Ok(Err(_)) => {
                break;
              }
              Err(_) => {
                log::warn!("WebSocket send timeout, closing connection due to slow client");
                break;
              }
            }
          }
        }
      }
    }
  });

  while let Some(msg) = ws_receiver.next().await {
    match msg {
      Ok(Message::Text(text)) => {
        let msg = RoomMessage::Message {
          user_id: user_id.clone(),
          content: text.to_string(),
        };
        let msg_json = match serde_json::to_string(&msg) {
          Ok(json) => json,
          Err(e) => {
            log::error!("Failed to serialize message: {}", e);
            continue;
          }
        };

        if let Err(e) = pubsub.publish(&room, &msg_json).await {
          log::error!("Failed to publish message: {}", e);
        }
      }
      Ok(Message::Close(_)) => {
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

  cancellation_token.cancel();

  let shutdown_timeout = std::time::Duration::from_secs(5);

  match tokio::time::timeout(shutdown_timeout, async {
    let (pubsub_result, ws_sender_result) =
      tokio::join!(&mut pubsub_task_handle, &mut ws_sender_task_handle);

    if let Err(e) = pubsub_result {
      if e.is_panic() {
        log::error!("PubSub task panicked: {:?}", e);
      }
    }

    if let Err(e) = ws_sender_result {
      if e.is_panic() {
        log::error!("WebSocket sender task panicked: {:?}", e);
      }
    }
  })
  .await
  {
    Err(_) => {
      log::warn!(
        "Task shutdown timed out after {:?}, forcefully aborting",
        shutdown_timeout
      );
      pubsub_task_handle.abort();
      ws_sender_task_handle.abort();

      tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Ok(_) => {}
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
