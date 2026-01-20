use std::{
  collections::HashMap,
  io,
  net::{IpAddr, SocketAddr},
  path::Path,
  str::FromStr,
  sync::Arc,
};

use axum::Router;
use clap::Parser;
use os_cli::{run::shutdown_signal, serve::serve};
use os_ui::helper::write_public_env_file;
use tokio_util::sync::CancellationToken;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "completions")]
use crate::cli::completions;
use crate::{
  cli::args::{CliArgs, CliCommand},
  config::{
    AppConfig, DOCUMENT_STORE_API_URL_PREFIX, FS_API_URL_PREFIX, OIDC_ADMIN_API_URL_PREFIX,
    OIDC_ADMIN_UI_URL_PREFIX, OIDC_API_URL_PREFIX, OIDC_UI_URL_PREFIX, SIGNALING_API_URL_PREFIX,
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
      let mut app_config = AppConfig::default();
      app_config.overwrite_dependencies();
      app_config
    }
  });

  LogTracer::init().map_err(|e| io::Error::other(format!("failed to init log tracer: {}", e)))?;

  let level =
    tracing::Level::from_str(&app_config.as_ref().log_level).unwrap_or(tracing::Level::DEBUG);
  let subscriber = tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
          "{level},axum::rejection=trace,sqlx=warn",
          level = level.as_str().to_lowercase()
        )
        .into()
      }),
    )
    .with(tracing_subscriber::fmt::layer());
  tracing::subscriber::set_global_default(subscriber)
    .map_err(|e| io::Error::other(format!("failed to set tracing subscriber: {}", e)))?;

  let cancellation_token = CancellationToken::new();

  let database_connection =
    match os_oidc_model::create_database_connection(&app_config.database).await {
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

  let mut oidc_ui_env_overrides = HashMap::default();

  oidc_ui_env_overrides.insert(
    "PUBLIC_OS_OIDC_API_URL".to_owned(),
    app_config.oidc_api.url(),
  );
  oidc_ui_env_overrides.insert(
    "PUBLIC_OS_OIDC_CLIENT_ID".to_owned(),
    app_config.oidc_api.client_id.clone(),
  );
  oidc_ui_env_overrides.insert(
    "PUBLIC_OS_OIDC_APPLICATION_URN".to_owned(),
    app_config.oidc_api.application_urn.clone(),
  );

  write_public_env_file(&app_config.oidc_ui, oidc_ui_env_overrides).await?;
  let oidc_ui_router = os_ui::router::create_router(&app_config.oidc_ui, Some(OIDC_UI_URL_PREFIX));

  let oidc_admin_openapi_router = os_oidc_admin::router::create_openapi_router(
    os_oidc_admin::router::entity::RouterState {
      database_connection: database_connection.clone(),
      config: Arc::new(app_config.oidc_admin_api.clone()),
    },
    Some(OIDC_ADMIN_API_URL_PREFIX),
  );

  let mut oidc_admin_ui_env_overrides = HashMap::default();

  oidc_admin_ui_env_overrides.insert(
    "PUBLIC_OS_OIDC_API_URL".to_owned(),
    app_config.oidc_api.url(),
  );
  oidc_admin_ui_env_overrides.insert(
    "PUBLIC_OS_OIDC_ADMIN_API_URL".to_owned(),
    app_config.oidc_admin_api.url(),
  );
  oidc_admin_ui_env_overrides.insert(
    "PUBLIC_OS_OIDC_ADMIN_CLIENT_ID".to_owned(),
    app_config.oidc_api.client_id.clone(),
  );
  oidc_admin_ui_env_overrides.insert(
    "PUBLIC_OS_OIDC_APPLICATION_URN".to_owned(),
    app_config.oidc_admin_api.oidc_application_urn.clone(),
  );

  write_public_env_file(&app_config.oidc_admin_ui, oidc_admin_ui_env_overrides).await?;
  let oidc_admin_ui_router =
    os_ui::router::create_router(&app_config.oidc_admin_ui, Some(OIDC_ADMIN_UI_URL_PREFIX));

  let pubsub = Arc::new(
    if let Some(redis_url) = &app_config.signaling_api.redis_url {
      match os_signaling::router::ws::pubsub::PubSub::new_redis(redis_url) {
        Ok(pubsub) => {
          log::info!("connected to redis pubsub at {}", redis_url);
          pubsub
        }
        Err(e) => {
          log::error!("failed to create redis pubsub client: {}", e);
          return Err(io::Error::other(e));
        }
      }
    } else {
      log::info!("using in-memory pubsub for signaling");
      os_signaling::router::ws::pubsub::PubSub::new_in_memory()
    },
  );

  let signaling_openapi_router = os_signaling::router::create_openapi_router(
    os_signaling::router::entity::RouterState {
      pubsub,
      config: Arc::new(app_config.signaling_api.clone()),
      cancellation_token: cancellation_token.clone(),
    },
    Some(SIGNALING_API_URL_PREFIX),
  );

  // for the service discovery endpoint we do not other endpoints like health checks, or version, etc.
  let service_discovery_openapi_router =
    os_service_discovery::router::discovery::router::create_router(
      os_service_discovery::router::entity::RouterState {
        config: Arc::new(app_config.service_discovery_api.clone()),
      },
    );

  let fs_openapi_router = os_fs::router::create_openapi_router(
    os_fs::router::entity::RouterState {
      config: Arc::new(app_config.fs_api.clone()),
    },
    Some(FS_API_URL_PREFIX),
  );

  let document_store_openapi_router = os_document_store::router::create_openapi_router(
    os_document_store::router::entity::RouterState {
      cancellation_token: cancellation_token.clone(),
      config: Arc::new(app_config.document_store_api.clone()),
    },
    Some(DOCUMENT_STORE_API_URL_PREFIX),
  );

  let router = Router::new()
    .merge(oidc_openapi_router)
    .merge(oidc_admin_openapi_router)
    .merge(signaling_openapi_router)
    .merge(fs_openapi_router)
    .merge(document_store_openapi_router)
    .merge(service_discovery_openapi_router)
    .layer(CorsLayer::very_permissive())
    .layer(TraceLayer::new_for_http())
    .merge(oidc_ui_router.reset_fallback())
    .merge(oidc_admin_ui_router.reset_fallback())
    .layer(CompressionLayer::new().gzip(app_config.server.gzip));

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

  let shutdown_timeout = std::time::Duration::from_secs(10);
  let mut command_handle = command_handle;
  tokio::select! {
    res = &mut command_handle => {
      match res {
        Ok(Ok(_)) => log::info!("server shutdown complete"),
        Ok(Err(e)) => log::error!("command error: {}", e),
        Err(e) => log::error!("join error: {}", e),
      }
    }
    _ = tokio::time::sleep(shutdown_timeout) => {
      log::warn!("server shutdown timed out after {:?}, aborting serve task", shutdown_timeout);
      command_handle.abort();
      tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
  }

  match os_oidc_model::connection::close_database_connection(database_connection).await {
    Ok(_) => log::info!("database connection closed"),
    Err(e) => log::error!("failed to close pool: {}", e),
  }

  Ok(())
}
