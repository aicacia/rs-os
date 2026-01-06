use std::{error::Error, io::Write, path::Path, sync::Arc};

use axum::Router;
use os_service_discovery::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState},
};

pub async fn setup() -> Result<(impl FnOnce(), Router, Arc<AppConfig>), Box<dyn Error>> {
  dotenvy::from_path("./.env.test").ok();

  let test_uuid = uuid::Uuid::new_v4();
  let test_dir = format!("tests/.tmp/{}", test_uuid);
  tokio::fs::create_dir_all(&test_dir).await?;

  let config_path = Path::new(&test_dir).join("config.json");
  {
    let mut config_file = std::fs::File::create(&config_path)?;
    config_file.write_all(serde_json::json!({}).to_string().as_bytes())?;
  }

  let app_config = Arc::new(AppConfig::try_from(config_path.as_path())?);

  let router = create_openapi_router(
    RouterState {
      config: app_config.clone(),
    },
    None,
  )
  .into();

  let teardown_test_dir = test_dir.clone();
  let teardown_fn = move || teardown(teardown_test_dir);

  Ok((teardown_fn, router, app_config))
}

fn teardown(test_dir: String) {
  tokio::task::block_in_place(move || {
    tokio::runtime::Handle::current().block_on(async move {
      match tokio::fs::remove_dir_all(&test_dir).await {
        Ok(_) => {}
        Err(e) => log::error!("failed to delete test directory {:?}: {}", test_dir, e),
      }
    });
  });
}
