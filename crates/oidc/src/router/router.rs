use axum::Router;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::router::{
  client,
  common::permissions::Permission,
  current_user,
  entity::RouterState,
  jwk, oidc,
  openapi::{self, utoipa::SecurityAddon},
  register, util,
};

#[derive(OpenApi)]
#[openapi(
  info(license(name = "MIT OR Apache-2.0", identifier = "https://spdx.org/licenses/MIT.html")),
  tags(
    (name = client::constants::TAG, description = client::constants::DESCRIPTION),
    (name = util::constants::TAG, description = util::constants::DESCRIPTION),
    (name = register::constants::TAG, description = register::constants::DESCRIPTION),
    (name = jwk::constants::TAG, description = jwk::constants::DESCRIPTION),
    (name = oidc::constants::TAG, description = oidc::constants::DESCRIPTION),
    (name = crate::router::openapi::constants::TAG, description = crate::router::openapi::constants::DESCRIPTION),
    (name = current_user::constants::TAG, description = current_user::constants::DESCRIPTION)
  ),
  components(
    schemas(
      Permission,
    )
  ),
  modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub fn create_router(state: RouterState, prefix_optional: Option<&str>) -> Router {
  let mut open_api_router = OpenApiRouter::with_openapi(ApiDoc::openapi());

  let open_api_routes = OpenApiRouter::new()
    .merge(client::router::create_router(state.clone()))
    .merge(util::router::create_router(state.clone()))
    .merge(register::router::create_router(state.clone()))
    .merge(jwk::router::create_router(state.clone()))
    .merge(oidc::router::create_router(state.clone()))
    .merge(current_user::router::create_router(state.clone()));

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
