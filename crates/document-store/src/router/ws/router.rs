use std::path::Path;

use axum::{
  extract::{
    Query, State,
    ws::{WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use os_api::{BasicClaims, HttpError, parse_token_data};
use tokio::sync::mpsc;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  ws::{
    constants::TAG,
    entity::{WSAuthorizationRequest, WSRequest},
    service::StorageSystem,
  },
};

#[utoipa::path(
  get,
  path = "/private",
  tags = [TAG],
  params(WSAuthorizationRequest),
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn private(
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

  let unique_key = format!(
    "client:{}:user:{}",
    if authorization_request.client {
      urlencoding::encode(&claims.client).to_string()
    } else {
      "anonymous".to_string()
    },
    if authorization_request.user {
      claims.user
    } else {
      0
    }
  );

  let shared_sync =
    match StorageSystem::get(Path::new(&state.config.storage.data_path), &unique_key).await {
      Ok(sync) => sync,
      Err(err) => return err.into_response(),
    };

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_ws(socket, shared_sync).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

#[utoipa::path(
  get,
  path = "/public",
  tags = [TAG],
  params(WSRequest),
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn public(
  State(state): State<RouterState>,
  Query(request): Query<WSRequest>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  let unique_key = format!("{}:public", urlencoding::encode(&request.storage));

  let shared_sync =
    match StorageSystem::get(Path::new(&state.config.storage.data_path), &unique_key).await {
      Ok(sync) => sync,
      Err(err) => return err.into_response(),
    };

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_ws(socket, shared_sync).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

async fn handle_ws(socket: WebSocket, shared_sync: StorageSystem) -> Result<(), HttpError> {
  let (mut ws_sender, ws_receiver) = socket.split();
  let (peer_sender, mut peer_receiver) = mpsc::unbounded_channel();

  let sender_task_handle = tokio::spawn(async move {
    while let Some(msg) = peer_receiver.recv().await {
      if let Err(err) = ws_sender.send(msg).await {
        log::error!("Failed to send message to WebSocket: {}", err);
        break;
      }
    }
  });

  shared_sync
    .handle_ws_messages(peer_sender, ws_receiver)
    .await;

  // Ensure the sender task terminates promptly when the receiver side ends.
  sender_task_handle.abort();
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;

  Ok(())
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(private))
    .routes(routes!(public))
    .with_state(state)
}
