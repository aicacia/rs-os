use axum::{extract::State, response::IntoResponse};
use utoipa::{
  Modify,
  openapi::{
    OpenApi as OpenApiSpec, RefOr, Schema, Server,
    security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
  },
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  error::{HttpError, INTERNAL_ERROR},
  middleware::AUTHORIZATION_HEADER,
  state::RouterState,
};

pub const TAG: &str = "openapi";
pub const DESCRIPTION: &str = "Open API endpoints";

pub struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        AUTHORIZATION_HEADER,
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .build(),
        ),
      );
    } else {
      log::warn!("OpenAPI components is None, cannot add security scheme");
    }
  }
}

pub struct ServersAddon {
  base_api_url: String,
}

impl ServersAddon {
  pub fn new(base_api_url: String) -> Self {
    Self { base_api_url }
  }
}

impl Modify for ServersAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    openapi
      .servers
      .get_or_insert(Vec::default())
      .push(Server::new(self.base_api_url.clone()));
  }
}

#[utoipa::path(
  get,
  path = "/openapi.json",
  tags = [TAG],
  responses(
    (status = 200, description = "OpenApi documenation"),
    (status = 500, description = "Internal server error", body = HttpError)
  )
)]
pub async fn get_openapi<C>(
  State((state, mut openapi)): State<(RouterState<C>, OpenApiSpec)>,
) -> impl IntoResponse
where
  C: crate::config::AppConfig,
{
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

pub fn create_router<C>(
  router_state: RouterState<C>,
  mut openapi_spec: OpenApiSpec,
  prefix_optional: Option<&str>,
) -> OpenApiRouter
where
  C: crate::config::AppConfig + 'static,
{
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
