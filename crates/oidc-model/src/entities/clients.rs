use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, Order, QueryOrder, QuerySelect, Set, TransactionTrait};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "clients")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub id: i64,
  #[sea_orm(default_value = "1")]
  pub active: i64,
  pub name: String,
  #[sea_orm(unique)]
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
  pub audience: String,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
  pub updated_at: i64,
  pub created_at: i64,
}

impl Model {
  pub fn is_active(&self) -> bool {
    self.active != 0
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::user_clients::Entity")]
  UserClients,
}

impl Related<super::user_clients::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserClients.def()
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    super::user_clients::Relation::Users.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::user_clients::Relation::Clients.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_client_by_id(
  db: &DatabaseConnection,
  client_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::Id.eq(client_id))
    .one(db)
    .await
}

pub async fn get_client_by_client_id(
  db: &DatabaseConnection,
  client_id: &str,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::ClientId.eq(client_id))
    .one(db)
    .await
}

pub async fn list_clients(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
  Entity::find()
    .order_by(Column::Id, Order::Asc)
    .all(db)
    .await
}

pub async fn get_distinct_grant_types(db: &DatabaseConnection) -> Result<Vec<String>, DbErr> {
  Entity::find()
    .select_only()
    .column(Column::GrantTypes)
    .into_tuple::<String>()
    .all(db)
    .await
}

pub async fn get_distinct_response_types(db: &DatabaseConnection) -> Result<Vec<String>, DbErr> {
  Entity::find()
    .select_only()
    .column(Column::ResponseTypes)
    .into_tuple::<String>()
    .all(db)
    .await
}

pub async fn get_distinct_scopes(db: &DatabaseConnection) -> Result<Vec<String>, DbErr> {
  Entity::find()
    .select_only()
    .column(Column::Scopes)
    .into_tuple::<String>()
    .all(db)
    .await
}

pub async fn get_distinct_auth_methods(db: &DatabaseConnection) -> Result<Vec<String>, DbErr> {
  Entity::find()
    .select_only()
    .column(Column::AuthMethod)
    .distinct()
    .into_tuple::<String>()
    .all(db)
    .await
}

pub async fn deactivate_client(
  db: &DatabaseConnection,
  client_id: &str,
) -> Result<Option<Model>, DbErr> {
  let mut active_model: ActiveModel = Entity::find()
    .filter(Column::ClientId.eq(client_id))
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("Client not found".to_string()))?
    .into();

  active_model.active = Set(0);
  active_model.updated_at = Set(chrono::Utc::now().timestamp());

  active_model.update(db).await.map(Some)
}

pub async fn upsert_client(
  db: &DatabaseConnection,
  client_upsert: ActiveModel,
  random_bytes_fn: impl Fn(usize) -> Vec<u8>,
) -> Result<(Model, bool), DbErr> {
  let txn = db.begin().await?;

  let client_id = client_upsert
    .client_id
    .clone()
    .into_value()
    .ok_or(DbErr::Custom("Client ID cannot be null".to_string()))?;

  let client_option = Entity::find()
    .filter(Column::ClientId.eq(client_id.clone()))
    .one(&txn)
    .await?;

  let result = if let Some(client) = client_option {
    let updated_client = update_client_internal(&txn, &client.client_id, client_upsert).await?;
    (updated_client, false)
  } else {
    let new_client = create_client_internal(&txn, client_upsert, random_bytes_fn).await?;
    (new_client, true)
  };

  txn.commit().await?;
  Ok(result)
}

async fn update_client_internal<C: ConnectionTrait>(
  db: &C,
  client_id: &str,
  client: ActiveModel,
) -> Result<Model, DbErr> {
  if client_id.is_empty() {
    return Err(DbErr::Custom("Client ID cannot be empty".to_string()));
  }

  let existing: ActiveModel = Entity::find()
    .filter(Column::ClientId.eq(client_id))
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("Client not found".to_string()))?
    .into();

  let mut active_model = existing;

  if let sea_orm::ActiveValue::Set(value) = &client.name {
    active_model.name = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.redirect_uris {
    active_model.redirect_uris = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.post_logout_redirect_uris {
    active_model.post_logout_redirect_uris = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.logo_uri {
    active_model.logo_uri = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.client_uri {
    active_model.client_uri = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.policy_uri {
    active_model.policy_uri = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.terms_of_service_uri {
    active_model.terms_of_service_uri = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.application_type {
    active_model.application_type = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.auth_method {
    active_model.auth_method = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.grant_types {
    active_model.grant_types = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.response_types {
    active_model.response_types = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.scopes {
    active_model.scopes = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.audience {
    active_model.audience = sea_orm::Set(value.clone());
  }
  if let sea_orm::ActiveValue::Set(value) = &client.access_token_expires_in_seconds {
    active_model.access_token_expires_in_seconds = sea_orm::Set(*value);
  }
  if let sea_orm::ActiveValue::Set(value) = &client.id_token_expires_in_seconds {
    active_model.id_token_expires_in_seconds = sea_orm::Set(*value);
  }
  if let sea_orm::ActiveValue::Set(value) = &client.refresh_expires_in_seconds {
    active_model.refresh_expires_in_seconds = sea_orm::Set(*value);
  }

  active_model.updated_at = Set(chrono::Utc::now().timestamp());

  active_model.update(db).await
}

async fn create_client_internal<C: ConnectionTrait>(
  db: &C,
  mut client: ActiveModel,
  random_bytes: impl Fn(usize) -> Vec<u8>,
) -> Result<Model, DbErr> {
  // Validate client_id before using client
  match &client.client_id {
    sea_orm::ActiveValue::Set(id) | sea_orm::ActiveValue::Unchanged(id) => {
      if id.is_empty() {
        return Err(DbErr::Custom("Client ID cannot be empty".to_string()));
      }
    }
    _ => return Err(DbErr::Custom("Client ID is required".to_string())),
  };

  let now = chrono::Utc::now().timestamp();
  client.client_secret = Set(hex::encode(random_bytes(64)));
  client.active = Set(1);
  client.created_at = Set(now);
  client.updated_at = Set(now);

  client.insert(db).await
}
