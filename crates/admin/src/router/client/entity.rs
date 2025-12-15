use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::core::helper::{json_to_string_vec, unordered_vec_equals};
use os_model::entities::clients;

#[derive(Serialize, ToSchema, Default)]
pub struct Client {
  pub id: i64,
  pub active: bool,
  pub name: String,
  pub client_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_secret: Option<String>,
  pub redirect_uris: Option<Vec<String>>,
  pub post_logout_redirect_uris: Option<Vec<String>>,
  pub logo_uri: Option<String>,
  pub client_uri: Option<String>,
  pub policy_uri: Option<String>,
  pub terms_of_service_uri: Option<String>,
  pub application_type: String,
  pub auth_method: String,
  pub grant_types: Vec<String>,
  pub response_types: Vec<String>,
  pub scopes: Vec<String>,
  pub audience: Vec<String>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<clients::Model> for Client {
  fn from(client_model: clients::Model) -> Self {
    Self {
      active: client_model.is_active(),
      id: client_model.id,
      name: client_model.name,
      client_id: client_model.client_id,
      client_secret: Some(client_model.client_secret),
      redirect_uris: client_model.redirect_uris.map(json_to_string_vec),
      post_logout_redirect_uris: client_model
        .post_logout_redirect_uris
        .map(json_to_string_vec),
      logo_uri: client_model.logo_uri,
      client_uri: client_model.client_uri,
      policy_uri: client_model.policy_uri,
      terms_of_service_uri: client_model.terms_of_service_uri,
      application_type: client_model.application_type,
      auth_method: client_model.auth_method,
      grant_types: json_to_string_vec(client_model.grant_types),
      response_types: json_to_string_vec(client_model.response_types),
      scopes: json_to_string_vec(client_model.scopes),
      audience: json_to_string_vec(client_model.audience),
      access_token_expires_in_seconds: client_model.access_token_expires_in_seconds,
      id_token_expires_in_seconds: client_model.id_token_expires_in_seconds,
      refresh_expires_in_seconds: client_model.refresh_expires_in_seconds,
      updated_at: DateTime::<Utc>::from_timestamp(client_model.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(client_model.created_at, 0).unwrap_or_default(),
    }
  }
}

#[derive(Deserialize, ToSchema)]
pub struct ClientUpsertRequest {
  pub name: String,
  pub client_id: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub redirect_uris: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub post_logout_redirect_uris: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub logo_uri: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub client_uri: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub policy_uri: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub terms_of_service_uri: Option<String>,
  pub application_type: String,
  pub auth_method: String,
  pub grant_types: Vec<String>,
  pub response_types: Vec<String>,
  pub scopes: Vec<String>,
  pub audience: Vec<String>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
}

pub fn client_upsert_request_changed(
  request: &ClientUpsertRequest,
  model: &clients::Model,
) -> bool {
  use crate::core::helper::string_vec_to_json;

  request.name != model.name
    || request.client_id != model.client_id
    || request.redirect_uris.as_ref().map(string_vec_to_json) != model.redirect_uris
    || request
      .post_logout_redirect_uris
      .as_ref()
      .map(string_vec_to_json)
      != model.post_logout_redirect_uris
    || request.logo_uri != model.logo_uri
    || request.client_uri != model.client_uri
    || request.policy_uri != model.policy_uri
    || request.terms_of_service_uri != model.terms_of_service_uri
    || request.application_type != model.application_type
    || request.auth_method != model.auth_method
    || string_vec_to_json(&request.grant_types) != model.grant_types
    || string_vec_to_json(&request.response_types) != model.response_types
    || string_vec_to_json(&request.scopes) != model.scopes
    || unordered_vec_equals(&request.audience, &json_to_string_vec(&model.audience))
    || request.access_token_expires_in_seconds != model.access_token_expires_in_seconds
    || request.id_token_expires_in_seconds != model.id_token_expires_in_seconds
    || request.refresh_expires_in_seconds != model.refresh_expires_in_seconds
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum ClientAuthorization {
  #[serde(rename = "authorization_code")]
  #[schema(title = "AuthorizationCode")]
  AuthorizationCode { code: String },
}

impl From<ClientUpsertRequest> for os_model::entities::clients::ActiveModel {
  fn from(request: ClientUpsertRequest) -> Self {
    use crate::core::helper::string_vec_to_json;
    use sea_orm::Set;

    let now = chrono::Utc::now().timestamp();
    Self {
      name: Set(request.name),
      client_id: Set(request.client_id),
      client_secret: Set(hex::encode(crate::core::encryption::random_bytes(64))),
      redirect_uris: Set(request.redirect_uris.as_ref().map(string_vec_to_json)),
      post_logout_redirect_uris: Set(
        request
          .post_logout_redirect_uris
          .as_ref()
          .map(string_vec_to_json),
      ),
      logo_uri: Set(request.logo_uri),
      client_uri: Set(request.client_uri),
      policy_uri: Set(request.policy_uri),
      terms_of_service_uri: Set(request.terms_of_service_uri),
      application_type: Set(request.application_type),
      auth_method: Set(request.auth_method),
      grant_types: Set(string_vec_to_json(&request.grant_types)),
      response_types: Set(string_vec_to_json(&request.response_types)),
      scopes: Set(string_vec_to_json(&request.scopes)),
      audience: Set(string_vec_to_json(&request.audience)),
      access_token_expires_in_seconds: Set(request.access_token_expires_in_seconds),
      id_token_expires_in_seconds: Set(request.id_token_expires_in_seconds),
      refresh_expires_in_seconds: Set(request.refresh_expires_in_seconds),
      active: Set(1),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    }
  }
}
