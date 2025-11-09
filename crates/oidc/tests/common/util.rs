use std::{error::Error, path::Path, str::FromStr, sync::Arc};

use axum::Router;
use os_oidc::{
  core::{
    config::app_config::AppConfig,
    jwk::helper::init_jwk,
    migrations::{POSTGRESQL_MIGRATOR, SQLITE_MIGRATOR},
  },
  router::{create_router, entity::RouterState},
};
use tokio::{fs::remove_file, runtime::Handle, task::block_in_place};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn setup()
-> Result<(impl FnOnce(), Router, Arc<AppConfig>, sqlx::AnyPool), Box<dyn Error>> {
  dotenvy::from_path("./.env.test").ok();
  sqlx::any::install_default_drivers();

  let mut config = AppConfig::try_from(Path::new("./config.test.json"))?;
  config.database.url = format!("sqlite:tests/.dbs/os-{}-test.db", uuid::Uuid::new_v4());
  let config = Arc::new(config);

  let level = tracing::Level::from_str(&config.log_level).unwrap_or(tracing::Level::DEBUG);
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

  let pool = os_db::pool::create(&config.database, &SQLITE_MIGRATOR, &POSTGRESQL_MIGRATOR).await?;

  let _ = init_jwk(&pool).await?;

  let cancellation_token = CancellationToken::new();
  let router = create_router(
    RouterState {
      pool: pool.clone(),
      config: config.clone(),
    },
    None,
  );

  let teardown_config = config.clone();
  let teardown_pool = pool.clone();
  let teardown_cancellation_token = cancellation_token.clone();
  let teardown_fn = move || teardown(teardown_config, teardown_pool, teardown_cancellation_token);

  Ok((teardown_fn, router, config, pool))
}

fn teardown(config: Arc<AppConfig>, pool: sqlx::AnyPool, cancellation_token: CancellationToken) {
  cancellation_token.cancel();

  block_in_place(move || {
    Handle::current().block_on(async move {
      match os_db::pool::close(pool).await {
        Ok(_) => {}
        Err(e) => log::error!("failed to close pool: {}", e),
      }
      if config.database.url.starts_with("sqlite:") {
        let path = Path::new(&config.database.url["sqlite:".len()..]);
        match remove_file(path).await {
          Ok(_) => {}
          Err(e) => log::error!("failed to delete file {:?}: {}", path, e),
        }
      }
    });
  });
}
