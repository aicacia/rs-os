use std::{
  io,
  net::{IpAddr, SocketAddr},
  path::Path,
  str::FromStr,
};

use clap::Parser;
use os_cli::{run::shutdown_signal, serve::serve};
use os_oidc_ui_embed::config::UIConfig;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::args::{CliArgs, CliCommand};
#[cfg(feature = "completions")]
use crate::cli::completions;

pub async fn run() -> io::Result<()> {
  let args = CliArgs::parse();

  match dotenvy::dotenv() {
    Ok(_) => {}
    Err(e) => return Err(io::Error::other(e)),
  }

  let ui_config = match UIConfig::try_from(Path::new(&args.config)) {
    Ok(app_config) => app_config,
    Err(e) => return Err(io::Error::other(e)),
  };

  let level = tracing::Level::from_str(&ui_config.log_level).unwrap_or(tracing::Level::DEBUG);
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
          "{}={level},tower_http={level},axum::rejection=trace",
          env!("CARGO_PKG_NAME"),
          level = level.as_str().to_lowercase()
        )
        .into()
      }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  let cancellation_token = CancellationToken::new();

  let run_serve = |host: Option<IpAddr>, port: Option<u16>| {
    let router = os_oidc_ui_embed::router::create_router(None);
    let addr = SocketAddr::from((
      host.unwrap_or(ui_config.server.host),
      port.unwrap_or(ui_config.server.port),
    ));

    tokio::spawn(serve(router, addr, cancellation_token.clone()))
  };

  let command_handle = match args.command {
    #[cfg(feature = "completions")]
    Some(CliCommand::Completions { shell }) => {
      tokio::task::spawn_blocking(move || Ok(completions::run(shell)))
    }
    Some(CliCommand::Serve { serve }) => run_serve(serve.host, serve.port),
    None => run_serve(None, None),
  };

  shutdown_signal(cancellation_token).await;

  match command_handle.await {
    Ok(_) => Ok(()),
    Err(e) => Err(io::Error::other(e)),
  }
}
