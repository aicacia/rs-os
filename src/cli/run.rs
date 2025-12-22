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
use tokio_util::sync::CancellationToken;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "completions")]
use crate::cli::completions;
use crate::{
  cli::args::{CliArgs, CliCommand},
  config::{
    ADMIN_API_URL_PREFIX, ADMIN_UI_URL_PREFIX, AppConfig, OIDC_API_URL_PREFIX, OIDC_UI_URL_PREFIX,
  },
};

pub async fn run() -> io::Result<()> {
  let args = CliArgs::parse();

  match dotenvy::dotenv() {
    Ok(_) => {}
    Err(e) => {
      eprintln!("failed to load .env file: {}", e);
    }
  }

  let app_config = Arc::new(match AppConfig::try_from(Path::new(&args.config)) {
    Ok(app_config) => app_config,
    Err(e) => {
      eprintln!("failed to load config {:?}: {}", args.config, e);
      return Err(io::Error::other(e));
    }
  });

  let level =
    tracing::Level::from_str(&app_config.as_ref().log_level).unwrap_or(tracing::Level::DEBUG);
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

  let database_connection = match os_model::create_database_connection(&app_config.database).await {
    Ok(db) => db,
    Err(e) => return Err(io::Error::other(e)),
  };

  os_oidc::core::model::init(&database_connection, &app_config.oidc_api).await?;

  let oidc_openapi_router = os_oidc::router::create_openapi_router(
    os_oidc::router::entity::RouterState {
      database_connection: database_connection.clone(),
      config: Arc::new(app_config.oidc_api.clone()),
    },
    Some(OIDC_API_URL_PREFIX),
  );
  let oidc_ui_router = os_oidc_ui::router::create_router(Some(OIDC_UI_URL_PREFIX));

  let admin_openapi_router = os_admin::router::create_openapi_router(
    os_admin::router::entity::RouterState {
      database_connection: database_connection.clone(),
      config: Arc::new(app_config.admin_api.clone()),
    },
    Some(ADMIN_API_URL_PREFIX),
  );
  let admin_ui_router = os_admin_ui::router::create_router(Some(ADMIN_UI_URL_PREFIX));

  let router = Router::new()
    .merge(oidc_openapi_router)
    .merge(admin_openapi_router)
    .layer(CorsLayer::very_permissive())
    .layer(TraceLayer::new_for_http())
    .merge(oidc_ui_router.reset_fallback())
    .merge(admin_ui_router.reset_fallback())
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
