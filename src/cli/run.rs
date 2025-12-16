use std::{
  io,
  net::{IpAddr, SocketAddr},
  path::Path,
  str::FromStr,
  sync::Arc,
};

use axum::Router;
use clap::Parser;
use os_cli::{run::shutdown_signal, serve::serve};
use os_model::entities::jwks::{create_jwk, generate_jwk, list_jwks};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "completions")]
use crate::cli::completions;
use crate::{
  cli::args::{CliArgs, CliCommand},
  config::{ADMIN_UI_URL_PREFIX, AppConfig, OIDC_API_URL_PREFIX, OIDC_UI_URL_PREFIX},
};

pub async fn run() -> io::Result<()> {
  let args = CliArgs::parse();

  match dotenvy::dotenv() {
    Ok(_) => {}
    Err(e) => return Err(io::Error::other(e)),
  }

  let app_config = Arc::new(match AppConfig::try_from(Path::new(&args.config)) {
    Ok(app_config) => app_config,
    Err(e) => {
      log::error!("failed to load config {:?}: {}", args.config, e);
      return Err(io::Error::other(e));
    }
  });

  let level =
    tracing::Level::from_str(&app_config.as_ref().log_level).unwrap_or(tracing::Level::DEBUG);
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

  let database_connection = match os_model::create_database_connection(&app_config.database).await {
    Ok(db) => db,
    Err(e) => return Err(io::Error::other(e)),
  };

  if list_jwks(&database_connection)
    .await
    .map_err(io::Error::other)?
    .is_empty()
  {
    let _ = create_jwk(
      &database_connection,
      generate_jwk(app_config.oidc_api.token.default_jwt_algorithm).map_err(io::Error::other)?,
    )
    .await
    .map_err(io::Error::other)?;
  }

  let oidc_router = os_oidc::router::create_router(
    os_oidc::router::entity::RouterState {
      database: database_connection.clone(),
      config: Arc::new(app_config.oidc_api.clone()),
    },
    Some(OIDC_API_URL_PREFIX),
  );
  let oidc_ui_router = os_oidc_ui_embed::router::create_router(Some(OIDC_UI_URL_PREFIX));

  let admin_ui_router = os_admin_ui_embed::router::create_router(Some(ADMIN_UI_URL_PREFIX));

  let router = Router::new()
    .merge(oidc_router)
    .merge(oidc_ui_router)
    .merge(admin_ui_router);

  let run_serve = |host: Option<IpAddr>, port: Option<u16>| {
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

  match command_handle.await {
    Ok(_) => {}
    Err(e) => {
      log::error!("command error: {}", e);
    }
  }

  match os_model::connection::close_database_connection(database_connection).await {
    Ok(_) => {}
    Err(e) => log::error!("failed to close pool: {}", e),
  }

  Ok(())
}
