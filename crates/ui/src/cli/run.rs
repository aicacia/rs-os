use std::{
  io,
  net::{IpAddr, SocketAddr},
  path::Path,
  str::FromStr,
};

use clap::Parser;
use os_cli::{run::shutdown_signal, serve::serve};
use tokio_util::sync::CancellationToken;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "completions")]
use crate::cli::completions;
use crate::{
  cli::args::{CliArgs, CliCommand},
  config::AppConfig,
  helper::write_public_env_file,
};

pub async fn run() -> io::Result<()> {
  let args = CliArgs::parse();

  match dotenvy::dotenv() {
    Ok(_) => {}
    Err(e) => {
      eprintln!("failed to load .env file: {}", e);
    }
  }

  let app_config = match AppConfig::try_from(Path::new(&args.config)) {
    Ok(app_config) => app_config,
    Err(e) => {
      eprintln!("failed to load config {:?}: {}", args.config, e);
      AppConfig::default()
    }
  };

  write_public_env_file(&app_config, Default::default()).await?;

  LogTracer::init().map_err(|e| io::Error::other(format!("failed to init log tracer: {}", e)))?;

  let level = tracing::Level::from_str(&app_config.log_level).unwrap_or(tracing::Level::DEBUG);
  let subscriber = tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
          "{level},axum::rejection=trace",
          level = level.as_str().to_lowercase()
        )
        .into()
      }),
    )
    .with(tracing_subscriber::fmt::layer());
  tracing::subscriber::set_global_default(subscriber)
    .map_err(|e| io::Error::other(format!("failed to set tracing subscriber: {}", e)))?;

  let cancellation_token = CancellationToken::new();

  let run_serve = |host: Option<IpAddr>, port: Option<u16>| {
    let router = crate::router::create_router(&app_config, None);
    let addr = SocketAddr::from((
      host.unwrap_or(app_config.server.host),
      port.unwrap_or(app_config.server.port),
    ));

    tokio::spawn(serve(router, addr, cancellation_token.clone()))
  };

  let command_handle = match args.command {
    #[cfg(feature = "completions")]
    Some(CliCommand::Completions { shell }) => tokio::task::spawn_blocking(move || {
      completions::run(shell);
      Ok(())
    }),
    Some(CliCommand::Serve { serve }) => run_serve(serve.host, serve.port),
    None => run_serve(None, None),
  };

  shutdown_signal(cancellation_token).await;

  let shutdown_timeout = std::time::Duration::from_secs(10);
  let mut command_handle = command_handle;
  tokio::select! {
    res = &mut command_handle => {
      match res {
        Ok(_) => Ok(()),
        Err(e) => Err(io::Error::other(e)),
      }
    }
    _ = tokio::time::sleep(shutdown_timeout) => {
      log::warn!("server shutdown timed out after {:?}, aborting serve task", shutdown_timeout);
      command_handle.abort();
      tokio::time::sleep(std::time::Duration::from_millis(100)).await;
      Ok(())
    }
  }
}
