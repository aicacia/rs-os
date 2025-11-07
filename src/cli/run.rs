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
use tokio::{runtime::Handle, task::block_in_place};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "completions")]
use crate::cli::completions;
use crate::{
  app_config::AppConfig,
  cli::args::{CliArgs, CliCommand},
};

pub async fn run() -> io::Result<()> {
  let args = CliArgs::parse();

  match dotenvy::dotenv() {
    Ok(_) => {}
    Err(e) => return Err(io::Error::other(e)),
  }

  let app_config = Arc::new(match AppConfig::try_from(Path::new(&args.config)) {
    Ok(app_config) => app_config,
    Err(e) => return Err(io::Error::other(e)),
  });

  let level =
    tracing::Level::from_str(&app_config.as_ref().oidc.log_level).unwrap_or(tracing::Level::DEBUG);
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
          "{}={level},tower_http={level},axum::rejection=trace,sqlx::query={level}",
          env!("CARGO_PKG_NAME"),
          level = level.as_str().to_lowercase()
        )
        .into()
      }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  let cancellation_token = CancellationToken::new();

  let (oidc_cleanup, oidc_router) =
    init_oidc(app_config.oidc.clone(), cancellation_token.clone()).await?;
  let oidc_ui_router = os_oidc_ui_embed::router::create_router(Some("/oidc"));
  let router = Router::new().merge(oidc_router).merge(oidc_ui_router);

  let run_serve = |host: Option<IpAddr>, port: Option<u16>| {
    let addr = SocketAddr::from((
      host.unwrap_or(app_config.server.host),
      port.unwrap_or(app_config.server.port),
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
    Ok(_) => {}
    Err(e) => {
      log::error!("command error: {}", e);
    }
  }

  oidc_cleanup();

  Ok(())
}

async fn init_oidc(
  app_config: os_oidc::core::config::app_config::AppConfig,
  cancellation_token: CancellationToken,
) -> io::Result<(impl FnOnce(), Router)> {
  let oidc_pool = match os_db::pool::create(
    &app_config.database,
    &os_oidc::core::migrations::SQLITE_MIGRATOR,
    &os_oidc::core::migrations::POSTGRESQL_MIGRATOR,
  )
  .await
  {
    Ok(pool) => pool,
    Err(e) => return Err(io::Error::other(e)),
  };

  match os_oidc::core::jwk::helper::init_jwk(&oidc_pool).await {
    Ok(_) => {}
    Err(e) => return Err(io::Error::other(e.to_string())),
  }

  let oidc_dynamic_app_config =
    match os_oidc::core::config::dynamic_app_config::DynamicAppConfig::with_background_updater(
      oidc_pool.clone(),
      cancellation_token.clone(),
    ) {
      Ok(dynamic_app_config) => dynamic_app_config,
      Err(e) => return Err(io::Error::other(e)),
    };

  let oidc_router = os_oidc::router::create_router(
    os_oidc::router::entity::RouterState {
      pool: oidc_pool.clone(),
      app_config: Arc::new(app_config),
      dynamic_app_config: oidc_dynamic_app_config,
    },
    Some("/oidc/api"),
  );

  let close_pool = move || {
    block_in_place(move || {
      Handle::current().block_on(async move {
        match os_db::pool::close(oidc_pool).await {
          Ok(_) => {}
          Err(e) => log::error!("failed to close pool: {}", e),
        }
      });
    });
  };

  Ok((close_pool, oidc_router))
}
