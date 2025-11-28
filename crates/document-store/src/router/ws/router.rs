use axum::{
  extract::{
    State,
    ws::{Message, WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use os_api::HttpError;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState, middleware::authorization::Authorization, ws::constants::TAG,
};

#[utoipa::path(
  get,
  path = "/ws",
  tags = [TAG],
  responses(
    (status = 101, description = "WebSocket connection established"),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Internal Server Error", body = HttpError),
  )
)]
async fn ws(
  State(state): State<RouterState>,
  _: Authorization<()>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  ws.on_upgrade(move |socket| async move {
    if let Err(err) = handle_ws(socket, state).await {
      log::error!("WebSocket error: {}", err);
    }
  })
}

async fn handle_ws(mut socket: WebSocket, _state: RouterState) -> Result<(), HttpError> {
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
        log::debug!("Received binary: {:?}", bin);
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
