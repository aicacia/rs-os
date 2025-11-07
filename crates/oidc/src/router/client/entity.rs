use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::model::client::sql::ClientSQLRow;

#[derive(Serialize, ToSchema, Default)]
pub struct Client {
  pub id: i64,
  pub active: bool,
  pub name: String,
  pub client_id: String,
  pub client_secret: String,
  pub redirect_uris: Option<String>,
  pub post_logout_redirect_uris: Option<String>,
  pub logo_uri: Option<String>,
  pub client_uri: Option<String>,
  pub policy_uri: Option<String>,
  pub terms_of_service_uri: Option<String>,
  pub application_type: String,
  pub auth_method: String,
  pub grant_types: String,
  pub response_types: String,
  pub scopes: String,
  pub audience: Option<String>,
  pub access_token_expires_in_seconds: DateTime<Utc>,
  pub id_token_expires_in_seconds: DateTime<Utc>,
  pub refresh_expires_in_seconds: DateTime<Utc>,
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
      client_secret: client_sql_row.client_secret,
      redirect_uris: client_sql_row.redirect_uris,
      post_logout_redirect_uris: client_sql_row.post_logout_redirect_uris,
      logo_uri: client_sql_row.logo_uri,
      client_uri: client_sql_row.client_uri,
      policy_uri: client_sql_row.policy_uri,
      terms_of_service_uri: client_sql_row.terms_of_service_uri,
      application_type: client_sql_row.application_type,
      auth_method: client_sql_row.auth_method,
      grant_types: client_sql_row.grant_types,
      response_types: client_sql_row.response_types,
      scopes: client_sql_row.scopes,
      audience: client_sql_row.audience,
      access_token_expires_in_seconds: DateTime::<Utc>::from_timestamp(
        client_sql_row.access_token_expires_in_seconds,
        0,
      )
      .unwrap_or_default(),
      id_token_expires_in_seconds: DateTime::<Utc>::from_timestamp(
        client_sql_row.id_token_expires_in_seconds,
        0,
      )
      .unwrap_or_default(),
      refresh_expires_in_seconds: DateTime::<Utc>::from_timestamp(
        client_sql_row.refresh_expires_in_seconds,
        0,
      )
      .unwrap_or_default(),
      updated_at: DateTime::<Utc>::from_timestamp(client_sql_row.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(client_sql_row.created_at, 0).unwrap_or_default(),
    }
  }
}
