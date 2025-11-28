use utoipa::{
  Modify,
  openapi::{
    Server,
    security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
  },
};

use crate::router::middleware::constants::AUTHORIZATION_HEADER;

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
