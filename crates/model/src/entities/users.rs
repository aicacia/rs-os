use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub id: i64,
  #[sea_orm(default_value = "1")]
  pub active: i64,
  #[sea_orm(unique)]
  pub username: String,
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
  #[sea_orm(has_many = "super::user_emails::Entity")]
  UserEmails,
  #[sea_orm(has_one = "super::user_infos::Entity")]
  UserInfos,
  #[sea_orm(has_many = "super::user_o_auth2_providers::Entity")]
  UserOAuth2Providers,
  #[sea_orm(has_many = "super::user_passwords::Entity")]
  UserPasswords,
  #[sea_orm(has_many = "super::user_phone_numbers::Entity")]
  UserPhoneNumbers,
  #[sea_orm(has_many = "super::user_roles::Entity")]
  UserRoles,
}

impl Related<super::user_clients::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserClients.def()
  }
}

impl Related<super::user_emails::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserEmails.def()
  }
}

impl Related<super::user_infos::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserInfos.def()
  }
}

impl Related<super::user_o_auth2_providers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserOAuth2Providers.def()
  }
}

impl Related<super::user_passwords::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserPasswords.def()
  }
}

impl Related<super::user_phone_numbers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserPhoneNumbers.def()
  }
}

impl Related<super::user_roles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserRoles.def()
  }
}

impl Related<super::clients::Entity> for Entity {
  fn to() -> RelationDef {
    super::user_clients::Relation::Clients.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::user_clients::Relation::Users.def().rev())
  }
}

impl Related<super::o_auth2_providers::Entity> for Entity {
  fn to() -> RelationDef {
    super::user_o_auth2_providers::Relation::OAuth2Providers.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::user_o_auth2_providers::Relation::Users.def().rev())
  }
}

impl Related<super::roles::Entity> for Entity {
  fn to() -> RelationDef {
    super::user_roles::Relation::Roles.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::user_roles::Relation::Users.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}

use sea_orm::{ConnectionTrait, Order, QueryOrder, Set, TransactionTrait, sea_query::Expr};
use std::collections::HashMap;

use super::{
  clients, permissions, roles, roles_permissions, user_clients, user_emails, user_infos,
  user_passwords, user_roles,
};

#[derive(Default)]
pub struct UserInfoUpdate {
  pub given_name: Option<String>,
  pub family_name: Option<String>,
  pub middle_name: Option<String>,
  pub nickname: Option<String>,
  pub profile_picture: Option<String>,
  pub website: Option<String>,
  pub gender: Option<String>,
  pub birthdate: Option<i64>,
  pub zone_info: Option<String>,
  pub locale: Option<String>,
  pub address: Option<String>,
}

pub async fn get_user_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Model>, DbErr> {
  Entity::find_by_id(id).one(db).await
}

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
  Entity::find()
    .filter(Column::Active.ne(0))
    .order_by(Column::CreatedAt, Order::Desc)
    .all(db)
    .await
}

pub async fn create_user(db: &DatabaseConnection, username: &str) -> Result<Model, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let user = ActiveModel {
    username: Set(username.to_owned()),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  user.insert(db).await
}

pub async fn update_user(
  db: &DatabaseConnection,
  user_id: i64,
  username: &str,
) -> Result<Model, DbErr> {
  let user = Entity::find_by_id(user_id)
    .filter(Column::Active.ne(0))
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

  let mut user: ActiveModel = user.into();
  user.username = Set(username.to_owned());
  user.updated_at = Set(chrono::Utc::now().timestamp());
  user.update(db).await
}

pub async fn delete_user(db: &DatabaseConnection, user_id: i64) -> Result<Option<Model>, DbErr> {
  let user = Entity::find_by_id(user_id)
    .filter(Column::Active.ne(0))
    .one(db)
    .await?;

  if let Some(user) = user {
    let mut user: ActiveModel = user.into();
    user.active = Set(0);
    user.updated_at = Set(chrono::Utc::now().timestamp());
    Ok(Some(user.update(db).await?))
  } else {
    Ok(None)
  }
}

pub async fn get_user_by_username_or_primary_email(
  db: &DatabaseConnection,
  username_or_email: &str,
) -> Result<Option<Model>, DbErr> {
  if let Some(user) = Entity::find()
    .filter(Column::Username.eq(username_or_email))
    .one(db)
    .await?
  {
    return Ok(Some(user));
  }

  if let Some(email) = user_emails::Entity::find()
    .filter(user_emails::Column::Email.eq(username_or_email))
    .filter(user_emails::Column::Primary.ne(0))
    .one(db)
    .await?
  {
    return Entity::find_by_id(email.user_id).one(db).await;
  }

  Ok(None)
}

pub async fn get_user_by_username(
  db: &DatabaseConnection,
  username: &str,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::Username.eq(username))
    .one(db)
    .await
}

pub async fn get_user_active_password_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<user_passwords::Model>, DbErr> {
  user_passwords::Entity::find()
    .filter(user_passwords::Column::Active.ne(0))
    .filter(user_passwords::Column::UserId.eq(user_id))
    .one(db)
    .await
}

async fn create_user_internal<C: ConnectionTrait>(
  db: &C,
  username: &str,
  user_info: UserInfoUpdate,
) -> Result<(Model, user_infos::Model), DbErr> {
  let now = chrono::Utc::now().timestamp();
  let user = ActiveModel {
    username: Set(username.to_owned()),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  let user = user.insert(db).await?;

  let nickname = user_info.nickname.unwrap_or_else(|| username.to_owned());

  let user_info_model = user_infos::ActiveModel {
    user_id: Set(user.id),
    given_name: Set(user_info.given_name),
    family_name: Set(user_info.family_name),
    middle_name: Set(user_info.middle_name),
    nickname: Set(Some(nickname)),
    profile_picture: Set(user_info.profile_picture),
    website: Set(user_info.website),
    gender: Set(user_info.gender),
    birthdate: Set(user_info.birthdate),
    zone_info: Set(user_info.zone_info),
    locale: Set(user_info.locale),
    address: Set(user_info.address),
    created_at: Set(now),
    updated_at: Set(now),
  };
  let user_info_model = user_info_model.insert(db).await?;

  Ok((user, user_info_model))
}

pub async fn create_user_with_password(
  db: &DatabaseConnection,
  username: &str,
  password: &str,
  encrypt_password_fn: impl Fn(&str) -> Result<String, Box<dyn std::error::Error>>,
) -> Result<Model, DbErr> {
  let encrypted_password = encrypt_password_fn(password)
    .map_err(|e| DbErr::Custom(format!("Failed to encrypt password: {}", e)))?;

  let txn = db.begin().await?;

  let (user, _user_info) = create_user_internal(&txn, username, UserInfoUpdate::default()).await?;

  let now = chrono::Utc::now().timestamp();
  let password_model = user_passwords::ActiveModel {
    user_id: Set(user.id),
    encrypted_password: Set(encrypted_password),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  password_model.insert(&txn).await?;

  txn.commit().await?;

  Ok(user)
}

pub async fn create_user_with_email_and_password(
  db: &DatabaseConnection,
  username: &str,
  email: &str,
  password: &str,
  user_info: UserInfoUpdate,
  encrypt_password_fn: impl Fn(&str) -> Result<String, Box<dyn std::error::Error>>,
) -> Result<Model, DbErr> {
  let encrypted_password = encrypt_password_fn(password)
    .map_err(|e| DbErr::Custom(format!("Failed to encrypt password: {}", e)))?;

  let txn = db.begin().await?;

  let (user, _user_info) = create_user_internal(&txn, username, user_info).await?;

  let now = chrono::Utc::now().timestamp();
  let password_model = user_passwords::ActiveModel {
    user_id: Set(user.id),
    encrypted_password: Set(encrypted_password),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  password_model.insert(&txn).await?;

  let email_model = user_emails::ActiveModel {
    user_id: Set(user.id),
    email: Set(email.to_owned()),
    verified: Set(0),
    primary: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  email_model.insert(&txn).await?;

  txn.commit().await?;

  Ok(user)
}

pub async fn get_user_client_by_client_id(
  db: &DatabaseConnection,
  user_id: i64,
  client_id: &str,
) -> Result<Option<user_clients::Model>, DbErr> {
  let client = clients::Entity::find()
    .filter(clients::Column::ClientId.eq(client_id))
    .one(db)
    .await?;

  if let Some(client) = client {
    user_clients::Entity::find()
      .filter(user_clients::Column::UserId.eq(user_id))
      .filter(user_clients::Column::ClientId.eq(client.client_id))
      .one(db)
      .await
  } else {
    Ok(None)
  }
}

pub async fn upsert_user_client(
  db: &DatabaseConnection,
  user_id: i64,
  client_id: &str,
  allowed_scopes: Vec<String>,
) -> Result<user_clients::Model, DbErr> {
  let client = clients::Entity::find()
    .filter(clients::Column::ClientId.eq(client_id))
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("Client not found".to_string()))?;

  let allowed_scopes_json = serde_json::to_string(&allowed_scopes)
    .map_err(|e| DbErr::Custom(format!("Failed to serialize scopes: {}", e)))?;

  let now = chrono::Utc::now().timestamp();

  if let Some(existing) = user_clients::Entity::find()
    .filter(user_clients::Column::UserId.eq(user_id))
    .filter(user_clients::Column::ClientId.eq(&client.client_id))
    .one(db)
    .await?
  {
    let mut active_model: user_clients::ActiveModel = existing.into();
    active_model.allowed_scopes = Set(allowed_scopes_json);
    active_model.updated_at = Set(now);
    active_model.update(db).await
  } else {
    let new_model = user_clients::ActiveModel {
      user_id: Set(user_id),
      client_id: Set(client.client_id),
      allowed_scopes: Set(allowed_scopes_json),
      created_at: Set(now),
      updated_at: Set(now),
    };
    new_model.insert(db).await
  }
}

pub async fn update_user_password(
  db: &DatabaseConnection,
  user_id: i64,
  password: &str,
  encrypt_password_fn: impl Fn(&str) -> Result<String, Box<dyn std::error::Error>>,
) -> Result<(), DbErr> {
  let encrypted_password = encrypt_password_fn(password)
    .map_err(|e| DbErr::Custom(format!("Failed to encrypt password: {}", e)))?;

  let txn = db.begin().await?;

  user_passwords::Entity::update_many()
    .col_expr(user_passwords::Column::Active, Expr::value(0))
    .filter(user_passwords::Column::UserId.eq(user_id))
    .filter(user_passwords::Column::Active.ne(0))
    .exec(&txn)
    .await?;

  let now = chrono::Utc::now().timestamp();
  let password_model = user_passwords::ActiveModel {
    user_id: Set(user_id),
    encrypted_password: Set(encrypted_password),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  password_model.insert(&txn).await?;

  txn.commit().await?;

  Ok(())
}

pub async fn get_user_roles_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<(user_roles::Model, Option<roles::Model>)>, DbErr> {
  user_roles::Entity::find()
    .filter(user_roles::Column::UserId.eq(user_id))
    .find_also_related(roles::Entity)
    .all(db)
    .await
}

pub async fn get_user_role_permissions_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<HashMap<i64, Vec<permissions::Model>>, DbErr> {
  let user_roles_list = user_roles::Entity::find()
    .filter(user_roles::Column::UserId.eq(user_id))
    .all(db)
    .await?;

  let role_ids: Vec<i64> = user_roles_list.iter().map(|ur| ur.role_id).collect();

  if role_ids.is_empty() {
    return Ok(HashMap::default());
  }

  let roles_permissions = roles_permissions::Entity::find()
    .filter(roles_permissions::Column::RoleId.is_in(role_ids))
    .find_also_related(permissions::Entity)
    .all(db)
    .await?;

  let mut permissions: HashMap<i64, Vec<permissions::Model>> = HashMap::default();

  for (roles_permission, permission_opt) in roles_permissions {
    if let Some(permission) = permission_opt {
      permissions
        .entry(roles_permission.role_id)
        .or_insert_with(Vec::new)
        .push(permission);
    }
  }

  Ok(permissions)
}

pub async fn assign_user_role(
  db: &DatabaseConnection,
  user_id: i64,
  role_id: i64,
) -> Result<roles::Model, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let user_role = user_roles::ActiveModel {
    user_id: Set(user_id),
    role_id: Set(role_id),
    created_at: Set(now),
    updated_at: Set(now),
  };

  let _ = user_role.insert(db).await;

  roles::Entity::find_by_id(role_id)
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("Role not found".to_string()))
}

pub async fn remove_user_role(
  db: &DatabaseConnection,
  user_id: i64,
  role_id: i64,
) -> Result<Option<roles::Model>, DbErr> {
  let result = user_roles::Entity::delete_many()
    .filter(user_roles::Column::UserId.eq(user_id))
    .filter(user_roles::Column::RoleId.eq(role_id))
    .exec(db)
    .await?;

  if result.rows_affected > 0 {
    roles::Entity::find_by_id(role_id).one(db).await
  } else {
    Ok(None)
  }
}
