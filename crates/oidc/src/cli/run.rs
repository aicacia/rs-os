use std::{
  io,
  net::{IpAddr, SocketAddr},
  path::Path,
  str::FromStr,
  sync::Arc,
};

use clap::Parser;
use os_cli::{run::shutdown_signal, serve::serve};
use os_model::entities::jwks::{create_jwk, generate_jwk, list_jwks};
use tokio_util::sync::CancellationToken;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "completions")]
use crate::cli::completions;
use crate::{
  cli::args::{CliArgs, CliCommand},
  config::AppConfig,
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
      generate_jwk(app_config.token.default_jwt_algorithm).map_err(io::Error::other)?,
    )
    .await
    .map_err(io::Error::other)?;
  }

  let open_api_router = crate::router::create_openapi_router(
    crate::router::entity::RouterState {
      database_connection: database_connection.clone(),
      config: app_config.clone(),
    },
    None,
  );

  let openapi_spec = open_api_router.get_openapi().clone();

  let base_api_url_app_config = app_config.clone();
  let base_api_url = move || base_api_url_app_config.base_api_url();

  let router = open_api_router
    .merge(os_api::openapi::create_router(
      base_api_url,
      openapi_spec,
      None,
    ))
    .layer(CorsLayer::very_permissive())
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new().gzip(app_config.server.gzip))
    .into();

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
