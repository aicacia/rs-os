use std::{error::Error, path::Path, sync::Arc};

use axum::Router;
use os_admin::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState},
};
use os_model::{
  create_database_connection,
  entities::jwks::{create_jwk, generate_jwk, list_jwks},
};
use tokio_util::sync::CancellationToken;

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

  let database_connection = create_database_connection(&app_config.database).await?;

  if list_jwks(&database_connection).await?.is_empty() {
    let _ = create_jwk(
      &database_connection,
      generate_jwk(app_config.token.default_jwt_algorithm)?,
    )
    .await?;
  }

  let cancellation_token = CancellationToken::new();
  let router = create_openapi_router(
    RouterState {
      database_connection: database_connection.clone(),
      config: app_config.clone(),
    },
    None,
  )
  .into();

  let teardown_config = app_config.clone();
  let teardown_database_connection = database_connection.clone();
  let teardown_cancellation_token = cancellation_token.clone();
  let teardown_fn = move || {
    teardown(
      teardown_config,
      teardown_database_connection,
      teardown_cancellation_token,
    )
  };

  Ok((teardown_fn, router, app_config, database_connection))
}

fn teardown(
  app_config: Arc<AppConfig>,
  database_connection: sea_orm::DatabaseConnection,
  cancellation_token: CancellationToken,
) {
  cancellation_token.cancel();

  tokio::task::block_in_place(move || {
    tokio::runtime::Handle::current().block_on(async move {
      match database_connection.close().await {
        Ok(_) => {}
        Err(e) => log::error!("failed to close database connection: {}", e),
      }

      if app_config.database.url.starts_with("sqlite:") {
        let path = Path::new(&app_config.database.url["sqlite:".len()..]);
        match tokio::fs::remove_file(path).await {
          Ok(_) => {}
          Err(e) => log::error!("failed to delete file {:?}: {}", path, e),
        }
      }
    });
  });
}
