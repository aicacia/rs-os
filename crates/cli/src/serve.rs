use std::{io, net::SocketAddr};

use axum::Router;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn serve(
  router: Router,
  addr: SocketAddr,
  cancellation_token: CancellationToken,
) -> io::Result<()> {
  let serve_shutdown_signal = async move {
    cancellation_token.cancelled().await;
  };

  let listener = TcpListener::bind(addr).await?;
  let local_addr = listener.local_addr()?;
  log::info!("listening on {}", local_addr);

  axum::serve(
    listener,
    router.into_make_service_with_connect_info::<SocketAddr>(),
  )
  .with_graceful_shutdown(serve_shutdown_signal)
  .await?;

  Ok(())
}
