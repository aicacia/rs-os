use std::{error::Error, io::Write, path::Path, sync::Arc};

use axum::Router;
use os_model::{
  create_database_connection,
  entities::jwks::{create_jwk, generate_jwk, list_jwks},
};
use os_oidc::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState},
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

  let test_uuid = uuid::Uuid::new_v4();
  let test_dir = format!("tests/.tmp/{}", test_uuid);
  tokio::fs::create_dir_all(&test_dir).await?;

  let config_path = Path::new(&test_dir).join("config.json");
  {
    let mut config_file = std::fs::File::create(&config_path)?;
    config_file.write_all(
      serde_json::json!({
        "database": {
          "url": format!("sqlite:{}/db.sqlite", test_dir)
        },
        "ui_url": "http://localhost:5173"
      })
      .to_string()
      .as_bytes(),
    )?;
  }

  let app_config = Arc::new(AppConfig::try_from(config_path.as_path())?);

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
      test_dir,
    )
  };

  Ok((teardown_fn, router, app_config, database_connection))
}

fn teardown(
  app_config: Arc<AppConfig>,
  database_connection: sea_orm::DatabaseConnection,
  cancellation_token: CancellationToken,
  test_dir: String,
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

      match tokio::fs::remove_dir_all(&test_dir).await {
        Ok(_) => {}
        Err(e) => log::error!("failed to delete test directory {:?}: {}", test_dir, e),
      }
    });
  });
}
