use axum::{extract::State, response::IntoResponse};
use utoipa::{
  Modify,
  openapi::{OpenApi as OpenApiSpec, RefOr, Schema},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  error::{HttpError, INTERNAL_ERROR},
  openapi::{constants::TAG, utoipa::ServersAddon},
};

#[utoipa::path(
  get,
  path = "/openapi.json",
  tags = [TAG],
  responses(
    (status = 200, description = "OpenApi documenation"),
    (status = 500, description = "Internal server error", body = HttpError)
  )
)]
pub async fn get_openapi(
  State((state, mut openapi)): State<(RouterState, OpenApiSpec)>,
) -> impl IntoResponse {
  let base_api_url = match state.config.base_api_url() {
    Ok(base_api_url) => base_api_url,
    Err(e) => {
      log::error!("Failed to get base_api_url from config: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  ServersAddon::new(base_api_url).modify(&mut openapi);
  axum::Json(openapi).into_response()
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
