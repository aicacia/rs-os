use axum::extract::State;
use utoipa::{
  Modify,
  openapi::{OpenApi as OpenApiSpec, RefOr, Schema},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  openapi::{constants::TAG, utoipa::ServersAddon},
};

#[utoipa::path(
  get,
  path = "/openapi.json",
  tags = [TAG],
  responses(
    (status = 200, description = "OpenApi documenation"),
  )
)]
pub async fn get_openapi(
  State((state, mut openapi)): State<(RouterState, OpenApiSpec)>,
) -> axum::Json<OpenApiSpec> {
  ServersAddon::new(&state.app_config, state.dynamic_app_config()).modify(&mut openapi);
  axum::Json(openapi)
}

pub fn create_router(
  router_state: RouterState,
  mut openapi_spec: OpenApiSpec,
  prefix_optional: Option<&str>,
) -> OpenApiRouter {
  let mut schemas = Vec::<(String, RefOr<Schema>)>::new();
  let (path, item, types) = routes!(@resolve_types get_openapi : schemas);

  openapi_spec.paths.add_path_operation(
    format!("{}{}", prefix_optional.unwrap_or_default(), path),
    types,
    item,
  );

  let mut openapi_router = OpenApiRouter::new()
    .routes(routes!(get_openapi))
    .with_state((router_state, openapi_spec));

  if let Some(prefix) = prefix_optional {
    openapi_router = OpenApiRouter::new().nest(prefix, openapi_router)
  }

  openapi_router
}
