use std::{error::Error, io::Write, path::Path, sync::Arc};

use axum::Router;
use os_signaling::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState, ws::pubsub::PubSub},
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
    config_file.write_all(serde_json::json!({}).to_string().as_bytes())?;
  }

  let app_config = Arc::new(AppConfig::try_from(config_path.as_path())?);

  let pubsub = Arc::new(if let Some(redis_url) = &app_config.redis_url {
    match PubSub::new_redis(redis_url) {
      Ok(pubsub) => pubsub,
      Err(e) => {
        log::error!("failed to create redis pubsub client: {}", e);
        return Err(e.into());
      }
    }
  } else {
    PubSub::new_in_memory()
  });

  let router = create_openapi_router(
    RouterState {
      config: app_config.clone(),
      pubsub,
      cancellation_token: CancellationToken::new(),
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
