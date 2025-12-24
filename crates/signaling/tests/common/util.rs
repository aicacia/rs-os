use std::{error::Error, path::Path, sync::Arc};

use axum::Router;
use os_model::{
  create_database_connection,
  entities::jwks::{create_jwk, generate_jwk, list_jwks},
};
use os_signaling::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState, ws::pubsub::PubSub},
};

pub async fn setup() -> Result<(Router, Arc<AppConfig>), Box<dyn Error>> {
  dotenvy::from_path("./.env.test").ok();

  let app_config = Arc::new({
    let mut app_config = AppConfig::try_from(Path::new("./config.test.json"))?;
    app_config.database.url = format!("sqlite:tests/.dbs/os-{}-test.db", uuid::Uuid::new_v4());
    app_config
  });

  let db = create_database_connection(&app_config.database).await?;

  if list_jwks(&db).await?.is_empty() {
    let _ = create_jwk(&db, generate_jwk(app_config.token.default_jwt_algorithm)?).await?;
  }

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
    },
    None,
  )
  .into();

  Ok((router, app_config))
}
