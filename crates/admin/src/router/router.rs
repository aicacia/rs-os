use os_api::SecurityAddon;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::router::{
  client, common::permissions::Permission, current_user, entity::RouterState, user, user_email,
  user_oauth2_provider, user_phone_number, user_role, util,
};

#[derive(OpenApi)]
#[openapi(
  info(license(name = "MIT OR Apache-2.0", identifier = "https://spdx.org/licenses/MIT.html")),
  components(
    schemas(
      Permission,
    )
  ),
  modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub fn create_openapi_router(state: RouterState, prefix_optional: Option<&str>) -> OpenApiRouter {
  let mut openapi_router = OpenApiRouter::with_openapi(ApiDoc::openapi());

  let app_router = OpenApiRouter::new()
    .merge(client::router::create_router(state.clone()))
    .merge(util::router::create_router(state.clone()))
    .merge(current_user::router::create_router(state.clone()))
    .merge(user::router::create_router(state.clone()))
    .merge(user_email::router::create_router(state.clone()))
    .merge(user_phone_number::router::create_router(state.clone()))
    .merge(user_oauth2_provider::router::create_router(state.clone()))
    .merge(user_role::router::create_router(state.clone()));

  if let Some(prefix) = prefix_optional {
    openapi_router = openapi_router.nest(prefix, app_router)
  } else {
    openapi_router = openapi_router.merge(app_router)
  }

  let openapi_spec = openapi_router.get_openapi().clone();

  let base_url_app_config = state.config.clone();
  let base_url = move || base_url_app_config.base_url();

  openapi_router.merge(os_api::openapi::create_router(
    base_url,
    openapi_spec,
    prefix_optional,
  ))
}
