use axum::Router;
use axum_embed::{FallbackBehavior, ServeEmbed};

use crate::embed::Assets;

pub fn create_router(prefix_optional: Option<&str>) -> Router {
  let embedded_assets = ServeEmbed::<Assets>::with_parameters(
    Some("index.html".to_owned()),
    FallbackBehavior::Ok,
    Some("index.html".to_owned()),
  );

  if let Some(prefix) = prefix_optional {
    Router::new().nest_service(prefix, embedded_assets)
  } else {
    Router::new().fallback_service(embedded_assets)
  }
}
