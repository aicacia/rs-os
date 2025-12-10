use hashbrown::HashMap;
use os_model::entities::{prelude::*, *};
use sea_orm::sea_query::Expr;
use sea_orm::*;

use crate::{
  core::{config::app_config::AppConfig, encryption::encrypt_password},
  model::rbac::orm::PermissionModel,
  router::common::permissions::Permission,
};

// Type aliases for backward compatibility
pub type UserModel = users::Model;
pub type UserInfoModel = user_infos::Model;
pub type UserPasswordModel = user_passwords::Model;
pub type UserEmailModel = user_emails::Model;
pub type UserPhoneNumberModel = user_phone_numbers::Model;
pub type UserOAuth2ProviderModel = user_o_auth2_providers::Model;
pub type UserClientModel = user_clients::Model;

// Helper trait for model extensions
pub trait UserModelExt {
  fn is_active(&self) -> bool;
}

impl UserModelExt for UserModel {
  fn is_active(&self) -> bool {
    self.active != 0
  }
}

pub trait UserEmailModelExt {
  fn is_verified(&self) -> bool;
  fn is_primary(&self) -> bool;
}

impl UserEmailModelExt for UserEmailModel {
  fn is_verified(&self) -> bool {
    self.verified != 0
  }
  fn is_primary(&self) -> bool {
    self.primary != 0
  }
}

pub trait UserPhoneNumberModelExt {
  fn is_verified(&self) -> bool;
  fn is_primary(&self) -> bool;
}

impl UserPhoneNumberModelExt for UserPhoneNumberModel {
  fn is_verified(&self) -> bool {
    self.verified != 0
  }
  fn is_primary(&self) -> bool {
    self.primary != 0
  }
}

pub async fn get_user_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<UserModel>, DbErr> {
  Users::find_by_id(id).one(db).await
}

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<UserModel>, DbErr> {
  Users::find()
    .filter(users::Column::Active.ne(0))
    .order_by_desc(users::Column::CreatedAt)
    .all(db)
    .await
}

pub async fn create_user(db: &DatabaseConnection, username: &str) -> Result<UserModel, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let user = users::ActiveModel {
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
) -> Result<UserModel, DbErr> {
  let user = Users::find_by_id(user_id)
    .filter(users::Column::Active.ne(0))
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

  let mut user: users::ActiveModel = user.into();
  user.username = Set(username.to_owned());
  user.updated_at = Set(chrono::Utc::now().timestamp());
  user.update(db).await
}

pub async fn delete_user(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<UserModel>, DbErr> {
  let user = Users::find_by_id(user_id)
    .filter(users::Column::Active.ne(0))
    .one(db)
    .await?;

  if let Some(user) = user {
    let mut user: users::ActiveModel = user.into();
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
) -> Result<Option<UserModel>, DbErr> {
  // First try to find by username
  if let Some(user) = Users::find()
    .filter(users::Column::Username.eq(username_or_email))
    .one(db)
    .await?
  {
    return Ok(Some(user));
  }

  // Then try to find by primary email
  if let Some(email) = UserEmails::find()
    .filter(user_emails::Column::Email.eq(username_or_email))
    .filter(user_emails::Column::Primary.ne(0))
    .one(db)
    .await?
  {
    return Users::find_by_id(email.user_id).one(db).await;
  }

  Ok(None)
}

pub async fn get_user_by_username(
  db: &DatabaseConnection,
  username: &str,
) -> Result<Option<UserModel>, DbErr> {
  Users::find()
    .filter(users::Column::Username.eq(username))
    .one(db)
    .await
}

pub async fn get_user_active_password_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<UserPasswordModel>, DbErr> {
  UserPasswords::find()
    .filter(user_passwords::Column::Active.ne(0))
    .filter(user_passwords::Column::UserId.eq(user_id))
    .one(db)
    .await
}

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

async fn create_user_internal<C: ConnectionTrait>(
  db: &C,
  username: &str,
  user_info: UserInfoUpdate,
) -> Result<(UserModel, UserInfoModel), DbErr> {
  let now = chrono::Utc::now().timestamp();
  let user = users::ActiveModel {
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
  app_config: &AppConfig,
  username: &str,
  password: &str,
) -> Result<UserModel, DbErr> {
  let encrypted_password = encrypt_password(app_config, password)
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
  app_config: &AppConfig,
  username: &str,
  email: &str,
  password: &str,
  user_info: UserInfoUpdate,
) -> Result<UserModel, DbErr> {
  let encrypted_password = encrypt_password(app_config, password)
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
) -> Result<Option<UserClientModel>, DbErr> {
  // First get the client to get its internal id
  let client = Clients::find()
    .filter(clients::Column::ClientId.eq(client_id))
    .one(db)
    .await?;

  if let Some(client) = client {
    UserClients::find()
      .filter(user_clients::Column::UserId.eq(user_id))
      .filter(user_clients::Column::ClientId.eq(client.id))
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
) -> Result<UserClientModel, DbErr> {
  // First get the client to get its internal id
  let client = Clients::find()
    .filter(clients::Column::ClientId.eq(client_id))
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("Client not found".to_string()))?;

  let allowed_scopes_json = serde_json::to_string(&allowed_scopes)
    .map_err(|e| DbErr::Custom(format!("Failed to serialize scopes: {}", e)))?;

  let now = chrono::Utc::now().timestamp();

  // Try to find existing user_client
  if let Some(existing) = UserClients::find()
    .filter(user_clients::Column::UserId.eq(user_id))
    .filter(user_clients::Column::ClientId.eq(client.id))
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
      client_id: Set(client.id.to_string()),
      allowed_scopes: Set(allowed_scopes_json),
      created_at: Set(now),
      updated_at: Set(now),
    };
    new_model.insert(db).await
  }
}

pub async fn get_user_info_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<UserInfoModel>, DbErr> {
  UserInfos::find_by_id(user_id).one(db).await
}

pub async fn update_user_info(
  db: &DatabaseConnection,
  user_id: i64,
  user_info: UserInfoUpdate,
) -> Result<UserInfoModel, DbErr> {
  let existing = UserInfos::find_by_id(user_id)
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("UserInfo not found".to_string()))?;

  let mut active_model: user_infos::ActiveModel = existing.into();

  if let Some(val) = user_info.given_name {
    active_model.given_name = Set(Some(val));
  }
  if let Some(val) = user_info.family_name {
    active_model.family_name = Set(Some(val));
  }
  if let Some(val) = user_info.middle_name {
    active_model.middle_name = Set(Some(val));
  }
  if let Some(val) = user_info.nickname {
    active_model.nickname = Set(Some(val));
  }
  if let Some(val) = user_info.profile_picture {
    active_model.profile_picture = Set(Some(val));
  }
  if let Some(val) = user_info.website {
    active_model.website = Set(Some(val));
  }
  if let Some(val) = user_info.gender {
    active_model.gender = Set(Some(val));
  }
  if let Some(val) = user_info.birthdate {
    active_model.birthdate = Set(Some(val));
  }
  if let Some(val) = user_info.zone_info {
    active_model.zone_info = Set(Some(val));
  }
  if let Some(val) = user_info.locale {
    active_model.locale = Set(Some(val));
  }
  if let Some(val) = user_info.address {
    active_model.address = Set(Some(val));
  }

  active_model.updated_at = Set(chrono::Utc::now().timestamp());
  active_model.update(db).await
}

pub async fn list_user_emails_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<UserEmailModel>, DbErr> {
  UserEmails::find()
    .filter(user_emails::Column::UserId.eq(user_id))
    .all(db)
    .await
}

pub async fn get_user_email_by_id(
  db: &DatabaseConnection,
  email_id: i64,
) -> Result<Option<UserEmailModel>, DbErr> {
  UserEmails::find_by_id(email_id).one(db).await
}

pub async fn create_user_email(
  db: &DatabaseConnection,
  user_id: i64,
  email: &str,
) -> Result<UserEmailModel, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let email_model = user_emails::ActiveModel {
    user_id: Set(user_id),
    email: Set(email.to_owned()),
    verified: Set(0),
    primary: Set(0),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  email_model.insert(db).await
}

pub async fn list_user_phone_numbers_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<UserPhoneNumberModel>, DbErr> {
  UserPhoneNumbers::find()
    .filter(user_phone_numbers::Column::UserId.eq(user_id))
    .all(db)
    .await
}

pub async fn get_user_phone_number_by_id(
  db: &DatabaseConnection,
  phone_id: i64,
) -> Result<Option<UserPhoneNumberModel>, DbErr> {
  UserPhoneNumbers::find_by_id(phone_id).one(db).await
}

pub async fn create_user_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
  phone_number: &str,
) -> Result<UserPhoneNumberModel, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let phone_model = user_phone_numbers::ActiveModel {
    user_id: Set(user_id),
    phone_number: Set(phone_number.to_owned()),
    verified: Set(0),
    primary: Set(0),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  phone_model.insert(db).await
}

pub async fn update_user_email_primary(
  db: &DatabaseConnection,
  user_id: i64,
  email_id: i64,
) -> Result<Option<UserEmailModel>, DbErr> {
  let txn = db.begin().await?;

  // First, unset all other primary emails for this user
  let now = chrono::Utc::now().timestamp();
  UserEmails::update_many()
    .col_expr(user_emails::Column::Primary, Expr::value(0))
    .col_expr(user_emails::Column::UpdatedAt, Expr::value(now))
    .filter(user_emails::Column::UserId.eq(user_id))
    .filter(user_emails::Column::Primary.ne(0))
    .exec(&txn)
    .await?;

  // Then set the specified email as primary
  let email = UserEmails::find_by_id(email_id)
    .filter(user_emails::Column::UserId.eq(user_id))
    .one(&txn)
    .await?;

  let result = if let Some(email) = email {
    let mut active_model: user_emails::ActiveModel = email.into();
    active_model.primary = Set(1);
    active_model.updated_at = Set(now);
    Some(active_model.update(&txn).await?)
  } else {
    None
  };

  txn.commit().await?;
  Ok(result)
}

pub async fn verify_user_email(
  db: &DatabaseConnection,
  user_id: i64,
  email_id: i64,
) -> Result<Option<UserEmailModel>, DbErr> {
  let email = UserEmails::find_by_id(email_id)
    .filter(user_emails::Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(email) = email {
    let mut active_model: user_emails::ActiveModel = email.into();
    active_model.verified = Set(1);
    active_model.updated_at = Set(chrono::Utc::now().timestamp());
    Ok(Some(active_model.update(db).await?))
  } else {
    Ok(None)
  }
}

pub async fn delete_user_email(
  db: &DatabaseConnection,
  user_id: i64,
  email_id: i64,
) -> Result<Option<UserEmailModel>, DbErr> {
  let email = UserEmails::find_by_id(email_id)
    .filter(user_emails::Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(email) = email {
    let email_clone = email.clone();
    UserEmails::delete_by_id(email_id).exec(db).await?;
    Ok(Some(email_clone))
  } else {
    Ok(None)
  }
}

pub async fn update_user_phone_number_primary(
  db: &DatabaseConnection,
  user_id: i64,
  phone_id: i64,
) -> Result<Option<UserPhoneNumberModel>, DbErr> {
  let txn = db.begin().await?;

  // First, unset all other primary phone numbers for this user
  let now = chrono::Utc::now().timestamp();
  UserPhoneNumbers::update_many()
    .col_expr(user_phone_numbers::Column::Primary, Expr::value(0))
    .col_expr(user_phone_numbers::Column::UpdatedAt, Expr::value(now))
    .filter(user_phone_numbers::Column::UserId.eq(user_id))
    .filter(user_phone_numbers::Column::Primary.ne(0))
    .exec(&txn)
    .await?;

  // Then set the specified phone number as primary
  let phone = UserPhoneNumbers::find_by_id(phone_id)
    .filter(user_phone_numbers::Column::UserId.eq(user_id))
    .one(&txn)
    .await?;

  let result = if let Some(phone) = phone {
    let mut active_model: user_phone_numbers::ActiveModel = phone.into();
    active_model.primary = Set(1);
    active_model.updated_at = Set(now);
    Some(active_model.update(&txn).await?)
  } else {
    None
  };

  txn.commit().await?;
  Ok(result)
}

pub async fn verify_user_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
  phone_id: i64,
) -> Result<Option<UserPhoneNumberModel>, DbErr> {
  let phone = UserPhoneNumbers::find_by_id(phone_id)
    .filter(user_phone_numbers::Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(phone) = phone {
    let mut active_model: user_phone_numbers::ActiveModel = phone.into();
    active_model.verified = Set(1);
    active_model.updated_at = Set(chrono::Utc::now().timestamp());
    Ok(Some(active_model.update(db).await?))
  } else {
    Ok(None)
  }
}

pub async fn delete_user_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
  phone_id: i64,
) -> Result<Option<UserPhoneNumberModel>, DbErr> {
  let phone = UserPhoneNumbers::find_by_id(phone_id)
    .filter(user_phone_numbers::Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(phone) = phone {
    let phone_clone = phone.clone();
    UserPhoneNumbers::delete_by_id(phone_id).exec(db).await?;
    Ok(Some(phone_clone))
  } else {
    Ok(None)
  }
}

// OAuth2 Provider functions
pub async fn get_user_oauth2_providers(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<(UserOAuth2ProviderModel, Option<o_auth2_providers::Model>)>, DbErr> {
  UserOAuth2Providers::find()
    .filter(user_o_auth2_providers::Column::UserId.eq(user_id))
    .find_also(UserOAuth2Providers, OAuth2Providers)
    .all(db)
    .await
}

pub async fn get_user_oauth2_provider_by_id(
  db: &DatabaseConnection,
  user_id: i64,
  provider_id: i64,
) -> Result<Option<(UserOAuth2ProviderModel, Option<o_auth2_providers::Model>)>, DbErr> {
  UserOAuth2Providers::find()
    .filter(user_o_auth2_providers::Column::UserId.eq(user_id))
    .filter(user_o_auth2_providers::Column::OAuth2ProviderId.eq(provider_id))
    .find_also(UserOAuth2Providers, OAuth2Providers)
    .one(db)
    .await
}

pub async fn link_user_oauth2_provider(
  db: &DatabaseConnection,
  user_id: i64,
  oauth2_provider_id: i64,
  _name: &str,
  email: &str,
) -> Result<UserOAuth2ProviderModel, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let model = user_o_auth2_providers::ActiveModel {
    user_id: Set(user_id),
    o_auth2_provider_id: Set(oauth2_provider_id),
    email: Set(email.to_owned()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  model.insert(db).await
}

pub async fn delete_user_oauth2_provider(
  db: &DatabaseConnection,
  user_id: i64,
  oauth2_provider_id: i64,
) -> Result<Option<UserOAuth2ProviderModel>, DbErr> {
  let provider = UserOAuth2Providers::find_by_id((user_id, oauth2_provider_id))
    .one(db)
    .await?;

  if let Some(provider) = provider {
    let provider_clone = provider.clone();
    UserOAuth2Providers::delete_by_id((user_id, oauth2_provider_id))
      .exec(db)
      .await?;
    Ok(Some(provider_clone))
  } else {
    Ok(None)
  }
}

// Role functions
pub async fn get_user_roles_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<(user_roles::Model, Option<roles::Model>)>, DbErr> {
  UserRoles::find()
    .filter(user_roles::Column::UserId.eq(user_id))
    .find_also(UserRoles, Roles)
    .all(db)
    .await
}

pub async fn get_user_role_permissions_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<HashMap<i64, Vec<PermissionModel>>, DbErr> {
  // First get all roles for the user
  let user_roles_list = UserRoles::find()
    .filter(user_roles::Column::UserId.eq(user_id))
    .all(db)
    .await?;

  let role_ids: Vec<i64> = user_roles_list.iter().map(|ur| ur.role_id).collect();

  if role_ids.is_empty() {
    return Ok(HashMap::default());
  }

  // Get all permissions for those roles
  let roles_permissions = RolesPermissions::find()
    .filter(roles_permissions::Column::RoleId.is_in(role_ids))
    .find_also(RolesPermissions, Permissions)
    .all(db)
    .await?;

  let mut permissions: HashMap<i64, Vec<PermissionModel>> = HashMap::default();

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

pub async fn get_user_permissions(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<Permission>, DbErr> {
  let role_permissions = get_user_role_permissions_by_user_id(db, user_id).await?;

  let mut permissions = Vec::new();
  for (_role_id, perms) in role_permissions {
    for perm in perms {
      if let Ok(permission) = perm.uri.parse::<Permission>() {
        if !permissions.contains(&permission) {
          permissions.push(permission);
        }
      }
    }
  }

  Ok(permissions)
}

pub async fn assign_user_role(
  db: &DatabaseConnection,
  user_id: i64,
  role_id: i64,
) -> Result<roles::Model, DbErr> {
  // First insert the user_role relationship
  let now = chrono::Utc::now().timestamp();
  let user_role = user_roles::ActiveModel {
    user_id: Set(user_id),
    role_id: Set(role_id),
    created_at: Set(now),
    updated_at: Set(now),
  };

  // Try to insert, ignore if it already exists
  let _ = user_role.insert(db).await;

  // Then fetch and return the role
  Roles::find_by_id(role_id)
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("Role not found".to_string()))
}

pub async fn remove_user_role(
  db: &DatabaseConnection,
  user_id: i64,
  role_id: i64,
) -> Result<Option<roles::Model>, DbErr> {
  // Delete the relationship
  let result = UserRoles::delete_many()
    .filter(user_roles::Column::UserId.eq(user_id))
    .filter(user_roles::Column::RoleId.eq(role_id))
    .exec(db)
    .await?;

  if result.rows_affected > 0 {
    // Fetch and return the role
    Roles::find_by_id(role_id).one(db).await
  } else {
    Ok(None)
  }
}

// Password functions
pub async fn update_user_password(
  db: &DatabaseConnection,
  app_config: &AppConfig,
  user_id: i64,
  password: &str,
) -> Result<(), DbErr> {
  let encrypted_password = encrypt_password(app_config, password)
    .map_err(|e| DbErr::Custom(format!("Failed to encrypt password: {}", e)))?;

  let txn = db.begin().await?;

  // Deactivate any existing active passwords for this user
  UserPasswords::update_many()
    .col_expr(user_passwords::Column::Active, Expr::value(0))
    .filter(user_passwords::Column::UserId.eq(user_id))
    .filter(user_passwords::Column::Active.ne(0))
    .exec(&txn)
    .await?;

  // Insert new active password row
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
