use std::{error::Error, path::Path, str::FromStr, sync::Arc};

use axum::Router;
use os_oidc::{
  core::{config::app_config::AppConfig, jwk::helper::init_jwk},
  router::{create_router, entity::RouterState},
};
use tokio::{fs::remove_file, runtime::Handle, task::block_in_place};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn setup() -> Result<
  (
    impl FnOnce(),
    Router,
    Arc<AppConfig>,
    sea_orm::DatabaseConnection,
  ),
  Box<dyn Error>,
> {
  dotenvy::from_path("./.env.test").ok();

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

  let db = sea_orm::Database::connect(&config.database.url).await?;

  // If you need to run migrations, use sea_orm_migration or a custom solution here
  // let _ = run_migrations(&db).await?;

  let _ = init_jwk(&db, &config).await?;

  let cancellation_token = CancellationToken::new();
  let router = create_router(
    RouterState {
      database: db.clone(),
      config: config.clone(),
    },
    None,
  );

  let teardown_config = config.clone();
  let teardown_db = db.clone();
  let teardown_cancellation_token = cancellation_token.clone();
  let teardown_fn = move || teardown(teardown_config, teardown_db, teardown_cancellation_token);

  Ok((teardown_fn, router, config, db))
}

fn teardown(
  config: Arc<AppConfig>,
  _db: sea_orm::DatabaseConnection,
  cancellation_token: CancellationToken,
) {
  cancellation_token.cancel();

  block_in_place(move || {
    Handle::current().block_on(async move {
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
