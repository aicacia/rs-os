use std::{error::Error, io, path::Path, str::FromStr, sync::Arc};

use axum::Router;
use os_model::{
  create_database_connection,
  entities::jwks::{create_jwk, generate_jwk, list_jwks},
};
use os_oidc::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState},
};
use tokio::{fs::remove_file, runtime::Handle, task::block_in_place};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::layer::SubscriberExt;

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

  let app_config = Arc::new({
    let mut app_config = AppConfig::try_from(Path::new("./config.test.json"))?;
    app_config.database.url = format!("sqlite:tests/.dbs/os-{}-test.db", uuid::Uuid::new_v4());
    app_config
  });

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

  let db = create_database_connection(&app_config.database).await?;

  if list_jwks(&db).await?.is_empty() {
    let _ = create_jwk(&db, generate_jwk(app_config.token.default_jwt_algorithm)?).await?;
  }

  let cancellation_token = CancellationToken::new();
  let router = create_openapi_router(
    RouterState {
      database_connection: db.clone(),
      config: app_config.clone(),
    },
    None,
  )
  .into();

  let teardown_config = app_config.clone();
  let teardown_db = db.clone();
  let teardown_cancellation_token = cancellation_token.clone();
  let teardown_fn = move || teardown(teardown_config, teardown_db, teardown_cancellation_token);

  Ok((teardown_fn, router, app_config, db))
}

fn teardown(
  app_config: Arc<AppConfig>,
  _db: sea_orm::DatabaseConnection,
  cancellation_token: CancellationToken,
) {
  cancellation_token.cancel();

  block_in_place(move || {
    Handle::current().block_on(async move {
      if app_config.database.url.starts_with("sqlite:") {
        let path = Path::new(&app_config.database.url["sqlite:".len()..]);
        match remove_file(path).await {
          Ok(_) => {}
          Err(e) => log::error!("failed to delete file {:?}: {}", path, e),
        }
      }
    });
  });
}
