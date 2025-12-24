use std::{error::Error, path::Path, sync::Arc};

use axum::Router;
use os_model::{
  create_database_connection,
  entities::jwks::{create_jwk, generate_jwk, list_jwks},
};
use os_signaling::{
  config::AppConfig,
  router::{create_openapi_router, entity::RouterState},
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

  let redis_client = redis::Client::open(app_config.redis_url.as_str())?;

  let router = create_openapi_router(
    RouterState {
      config: app_config.clone(),
      redis_client,
    },
    None,
  )
  .into();

  Ok((router, app_config))
}
