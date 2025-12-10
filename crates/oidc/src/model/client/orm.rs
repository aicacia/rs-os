use os_model::entities::{prelude::*, *};
use sea_orm::*;

use crate::core::encryption::random_bytes;

// Type alias for backward compatibility
pub type ClientModel = clients::Model;

// Helper trait for model extensions
pub trait ClientModelExt {
  fn is_active(&self) -> bool;
}

impl ClientModelExt for ClientModel {
  fn is_active(&self) -> bool {
    self.active != 0
  }
}

pub async fn get_client_by_client_id(
  db: &DatabaseConnection,
  client_id: &str,
) -> Result<Option<ClientModel>, DbErr> {
  Clients::find()
    .filter(clients::Column::ClientId.eq(client_id))
    .one(db)
    .await
}

pub async fn list_clients(db: &DatabaseConnection) -> Result<Vec<ClientModel>, DbErr> {
  Clients::find()
    .order_by_asc(clients::Column::Id)
    .all(db)
    .await
}

pub async fn deactivate_client(
  db: &DatabaseConnection,
  client_id: &str,
) -> Result<Option<ClientModel>, DbErr> {
  let client = Clients::find()
    .filter(clients::Column::ClientId.eq(client_id))
    .one(db)
    .await?;

  if let Some(client) = client {
    let mut active_model: clients::ActiveModel = client.into();
    active_model.active = Set(0);
    active_model.updated_at = Set(chrono::Utc::now().timestamp());
    Ok(Some(active_model.update(db).await?))
  } else {
    Ok(None)
  }
}

pub struct ClientCommon {
  pub name: String,
  pub client_id: String,
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
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
}

impl PartialEq<ClientModel> for ClientCommon {
  fn eq(&self, other: &ClientModel) -> bool {
    self.name == other.name
      && self.client_id == other.client_id
      && self.redirect_uris == other.redirect_uris
      && self.post_logout_redirect_uris == other.post_logout_redirect_uris
      && self.logo_uri == other.logo_uri
      && self.policy_uri == other.policy_uri
      && self.terms_of_service_uri == other.terms_of_service_uri
      && self.application_type == other.application_type
      && self.auth_method == other.auth_method
      && self.grant_types == other.grant_types
      && self.response_types == other.response_types
      && self.scopes == other.scopes
      && self.audience == other.audience
      && self.access_token_expires_in_seconds == other.access_token_expires_in_seconds
      && self.id_token_expires_in_seconds == other.id_token_expires_in_seconds
      && self.refresh_expires_in_seconds == other.refresh_expires_in_seconds
  }
}

pub async fn upsert_client(
  db: &DatabaseConnection,
  client_upsert: ClientCommon,
) -> Result<(ClientModel, bool), DbErr> {
  let txn = db.begin().await?;

  let client_option = Clients::find()
    .filter(clients::Column::ClientId.eq(&client_upsert.client_id))
    .one(&txn)
    .await?;

  let result = if let Some(client) = client_option {
    if client_upsert != client {
      let updated_client = update_client_internal(&txn, &client.client_id, client_upsert).await?;
      (updated_client, false)
    } else {
      (client, false)
    }
  } else {
    let new_client = create_client_internal(&txn, client_upsert).await?;
    (new_client, true)
  };

  txn.commit().await?;
  Ok(result)
}

async fn update_client_internal<C: ConnectionTrait>(
  db: &C,
  client_id: &str,
  client: ClientCommon,
) -> Result<ClientModel, DbErr> {
  let existing = Clients::find()
    .filter(clients::Column::ClientId.eq(client_id))
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("Client not found".to_string()))?;

  let mut active_model: clients::ActiveModel = existing.into();
  active_model.name = Set(client.name);
  active_model.redirect_uris = Set(client.redirect_uris);
  active_model.post_logout_redirect_uris = Set(client.post_logout_redirect_uris);
  active_model.logo_uri = Set(client.logo_uri);
  active_model.client_uri = Set(client.client_uri);
  active_model.policy_uri = Set(client.policy_uri);
  active_model.terms_of_service_uri = Set(client.terms_of_service_uri);
  active_model.application_type = Set(client.application_type);
  active_model.auth_method = Set(client.auth_method);
  active_model.grant_types = Set(client.grant_types);
  active_model.response_types = Set(client.response_types);
  active_model.scopes = Set(client.scopes);
  active_model.audience = Set(client.audience);
  active_model.access_token_expires_in_seconds = Set(client.access_token_expires_in_seconds);
  active_model.id_token_expires_in_seconds = Set(client.id_token_expires_in_seconds);
  active_model.refresh_expires_in_seconds = Set(client.refresh_expires_in_seconds);
  active_model.updated_at = Set(chrono::Utc::now().timestamp());

  active_model.update(db).await
}

async fn create_client_internal<C: ConnectionTrait>(
  db: &C,
  client: ClientCommon,
) -> Result<ClientModel, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let new_client = clients::ActiveModel {
    name: Set(client.name),
    client_id: Set(client.client_id),
    client_secret: Set(hex::encode(random_bytes(64))),
    redirect_uris: Set(client.redirect_uris),
    post_logout_redirect_uris: Set(client.post_logout_redirect_uris),
    logo_uri: Set(client.logo_uri),
    client_uri: Set(client.client_uri),
    policy_uri: Set(client.policy_uri),
    terms_of_service_uri: Set(client.terms_of_service_uri),
    application_type: Set(client.application_type),
    auth_method: Set(client.auth_method),
    grant_types: Set(client.grant_types),
    response_types: Set(client.response_types),
    scopes: Set(client.scopes),
    audience: Set(client.audience),
    access_token_expires_in_seconds: Set(client.access_token_expires_in_seconds),
    id_token_expires_in_seconds: Set(client.id_token_expires_in_seconds),
    refresh_expires_in_seconds: Set(client.refresh_expires_in_seconds),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };

  new_client.insert(db).await
}
