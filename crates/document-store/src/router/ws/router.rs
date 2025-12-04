use std::path::Path;

use axum::{
  extract::{
    Query, State,
    ws::{Message, WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use os_api::{BasicClaims, HttpError};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::ws::{
  entity::{FromServerMessage, PeerMessage, PeerMetadata},
  service::SharedStorage,
};
use crate::router::{
  entity::RouterState,
  middleware::authorization::parse_token_data,
  ws::{
    constants::TAG,
    entity::{FromClientMessage, WSAuthorizationRequest},
  },
};

#[utoipa::path(
  get,
  path = "/ws",
  tags = [TAG],
  params(WSAuthorizationRequest),
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn ws(
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

  let sub = claims.sub;
  let aud = claims.aud;

  let shared =
    match SharedStorage::get(Path::new(&state.config.storage.data_path), &aud, &sub).await {
      Ok(s) => s,
      Err(err) => return err.into_response(),
    };

  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_ws(socket, shared).await {
      log::error!("WebSocket error: {}", err);
    }
  })
  .into_response()
}

async fn handle_ws(mut socket: WebSocket, storage: SharedStorage) -> Result<(), HttpError> {
  while let Some(result) = socket.recv().await {
    let msg = match result {
      Ok(m) => m,
      Err(e) => {
        log::error!("WebSocket receive error: {}", e);
        continue;
      }
    };

    match msg {
      Message::Text(text) => {
        log::debug!("Received text: {}", text);
      }
      Message::Binary(bin) => {
        let client_msg: FromClientMessage = match ciborium::from_reader(bin.as_ref()) {
          Ok(msg) => msg,
          Err(err) => {
            log::error!("Failed to deserialize CBOR message: {}", err);
            continue;
          }
        };
        match client_msg {
          FromClientMessage::Join(join_message) => {
            let mut cbor_message = Vec::new();
            match ciborium::into_writer(
              &FromServerMessage::Peer(PeerMessage {
                sender_id: storage.peer_id(),
                peer_metadata: PeerMetadata {
                  storage_id: join_message.peer_metadata.storage_id.clone(),
                  is_ephemeral: false,
                },
                target_id: join_message.sender_id.clone(),
              }),
              &mut cbor_message,
            ) {
              Ok(()) => {}
              Err(err) => {
                log::error!("Failed to serialize CBOR message: {}", err);
                continue;
              }
            }
            match socket.send(Message::Binary(cbor_message.into())).await {
              Ok(_) => {}
              Err(err) => {
                log::error!("Failed to send message over WebSocket: {}", err);
              }
            }
          }
          FromClientMessage::Sync(sync_message) => {
            log::debug!("Received Sync message: {:?}", sync_message);
            continue;
          }
          FromClientMessage::Ephemeral(ephemeral_message) => {
            log::debug!("Received Ephemeral message: {:?}", ephemeral_message);
            continue;
          }
          FromClientMessage::Request(request_message) => {
            log::debug!("Received Request message: {:?}", request_message);
            continue;
          }
          FromClientMessage::DocumentUnavailable(document_unavailable_message) => {
            log::debug!(
              "Received DocumentUnavailable message: {:?}",
              document_unavailable_message
            );
            continue;
          }
          FromClientMessage::RemoteSubscriptionControl(remote_subscription_control_message) => {
            log::debug!(
              "Received RemoteSubscriptionControl message: {:?}",
              remote_subscription_control_message
            );
            continue;
          }
          FromClientMessage::RemoteHeadsChanged(remote_heads_changed) => {
            log::debug!(
              "Received RemoteHeadsChanged message: {:?}",
              remote_heads_changed
            );
            continue;
          }
        }
      }
      Message::Ping(p) => {
        log::debug!("Received ping: {:?}", p);
      }
      Message::Pong(p) => {
        log::debug!("Received pong: {:?}", p);
      }
      Message::Close(frame) => {
        log::debug!("Received close: {:?}", frame);
        break;
      }
    }
  }
  Ok(())
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new().routes(routes!(ws)).with_state(state)
}
