use axum::Router;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::router::{
  entity::RouterState,
  openapi::{self, utoipa::SecurityAddon},
  util,
};

#[derive(OpenApi)]
#[openapi(
  info(license(name = "MIT OR Apache-2.0", identifier = "https://spdx.org/licenses/MIT.html")),
  tags(
    (name = util::constants::TAG, description = util::constants::DESCRIPTION),
    (name = crate::router::openapi::constants::TAG, description = crate::router::openapi::constants::DESCRIPTION)
  ),
  modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub fn create_router(state: RouterState, prefix_optional: Option<&str>) -> Router {
  let mut open_api_router = OpenApiRouter::with_openapi(ApiDoc::openapi());

  let open_api_routes = OpenApiRouter::new().merge(util::router::create_router(state.clone()));

  if let Some(prefix) = prefix_optional {
    open_api_router = open_api_router.nest(prefix, open_api_routes);
  }

  let openapi_spec = open_api_router.get_openapi().clone();
  open_api_router
    .merge(openapi::router::create_router(
      state.clone(),
      openapi_spec,
      prefix_optional,
    ))
    .layer(CorsLayer::very_permissive())
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new().gzip(state.config.server.gzip))
    .into()
}
