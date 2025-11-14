use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
  core::helper::json_to_string_vec,
  model::client::sql::{ClientSQLCommon, ClientSQLRow},
};

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
  pub audience: Option<Vec<String>>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<ClientSQLRow> for Client {
  fn from(client_sql_row: ClientSQLRow) -> Self {
    Self {
      active: client_sql_row.is_active(),
      id: client_sql_row.id,
      name: client_sql_row.name,
      client_id: client_sql_row.client_id,
      client_secret: Some(client_sql_row.client_secret),
      redirect_uris: client_sql_row.redirect_uris.map(json_to_string_vec),
      post_logout_redirect_uris: client_sql_row
        .post_logout_redirect_uris
        .map(json_to_string_vec),
      logo_uri: client_sql_row.logo_uri,
      client_uri: client_sql_row.client_uri,
      policy_uri: client_sql_row.policy_uri,
      terms_of_service_uri: client_sql_row.terms_of_service_uri,
      application_type: client_sql_row.application_type,
      auth_method: client_sql_row.auth_method,
      grant_types: json_to_string_vec(client_sql_row.grant_types),
      response_types: json_to_string_vec(client_sql_row.response_types),
      scopes: json_to_string_vec(client_sql_row.scopes),
      audience: client_sql_row.audience.map(json_to_string_vec),
      access_token_expires_in_seconds: client_sql_row.access_token_expires_in_seconds,
      id_token_expires_in_seconds: client_sql_row.id_token_expires_in_seconds,
      refresh_expires_in_seconds: client_sql_row.refresh_expires_in_seconds,
      updated_at: DateTime::<Utc>::from_timestamp(client_sql_row.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(client_sql_row.created_at, 0).unwrap_or_default(),
    }
  }
}

#[derive(Deserialize, ToSchema, Default)]
pub struct ClientUpsertRequest {
  pub name: String,
  pub client_id: String,
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
  pub audience: Option<Vec<String>>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
}

impl Into<ClientSQLCommon> for ClientUpsertRequest {
  fn into(self) -> ClientSQLCommon {
    ClientSQLCommon {
      name: self.name,
      client_id: self.client_id,
      redirect_uris: self
        .redirect_uris
        .map(|v| serde_json::to_string(&v).unwrap()),
      post_logout_redirect_uris: self
        .post_logout_redirect_uris
        .map(|v| serde_json::to_string(&v).unwrap()),
      logo_uri: self.logo_uri,
      client_uri: self.client_uri,
      policy_uri: self.policy_uri,
      terms_of_service_uri: self.terms_of_service_uri,
      application_type: self.application_type,
      auth_method: self.auth_method,
      grant_types: serde_json::to_string(&self.grant_types).unwrap(),
      response_types: serde_json::to_string(&self.response_types).unwrap(),
      scopes: serde_json::to_string(&self.scopes).unwrap(),
      audience: self.audience.map(|v| serde_json::to_string(&v).unwrap()),
      access_token_expires_in_seconds: self.access_token_expires_in_seconds,
      id_token_expires_in_seconds: self.id_token_expires_in_seconds,
      refresh_expires_in_seconds: self.refresh_expires_in_seconds,
    }
  }
}

#[derive(Serialize, ToSchema, Default)]
pub struct ClientAllowed {
  pub allowed_scopes: Vec<String>,
}
