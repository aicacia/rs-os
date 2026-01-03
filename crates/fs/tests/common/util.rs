use std::{error::Error, io::Write, path::Path, sync::Arc};

use axum::Router;
use os_fs::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState},
};
use tokio_util::sync::CancellationToken;

pub async fn setup() -> Result<(impl FnOnce(), Router, Arc<AppConfig>), Box<dyn Error>> {
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
        }
      })
      .to_string()
      .as_bytes(),
    )?;
  }

  let app_config = Arc::new(AppConfig::try_from(config_path.as_path())?);

  let cancellation_token = CancellationToken::new();
  let router = create_openapi_router(
    RouterState {
      config: app_config.clone(),
    },
    None,
  )
  .into();

  let teardown_config = app_config.clone();
  let teardown_cancellation_token = cancellation_token.clone();
  let teardown_test_dir = test_dir.clone();
  let teardown_fn = move || {
    teardown(
      teardown_config,
      teardown_cancellation_token,
      teardown_test_dir,
    )
  };

  Ok((teardown_fn, router, app_config))
}

fn teardown(_app_config: Arc<AppConfig>, cancellation_token: CancellationToken, test_dir: String) {
  cancellation_token.cancel();

  tokio::task::block_in_place(move || {
    tokio::runtime::Handle::current().block_on(async move {
      match tokio::fs::remove_dir_all(&test_dir).await {
        Ok(_) => {}
        Err(e) => log::error!("failed to delete test directory {:?}: {}", test_dir, e),
      }
    });
  });
}
