use utoipa::{
  Modify,
  openapi::{
    Server,
    security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
  },
};

use crate::{core::config::app_config::AppConfig, router::common::constants::AUTHORIZATION_HEADER};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    let components = openapi.components.as_mut().unwrap();
    components.add_security_scheme(
      AUTHORIZATION_HEADER,
      SecurityScheme::Http(
        HttpBuilder::new()
          .scheme(HttpAuthScheme::Bearer)
          .bearer_format("JWT")
          .build(),
      ),
    );
  }
}

pub struct ServersAddon<'a> {
  app_config: &'a AppConfig,
}

impl<'a> ServersAddon<'a> {
  pub fn new(app_config: &'a AppConfig) -> Self {
    Self { app_config }
  }
}

impl<'a> Modify for ServersAddon<'a> {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    openapi
      .servers
      .get_or_insert(Vec::default())
      .push(Server::new(self.app_config.base_api_url()));
  }
}
