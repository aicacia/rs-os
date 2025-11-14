use hashbrown::HashMap;

use os_db::pool::run_transaction;

use crate::{
  core::{
    config::app_config::AppConfig,
    encryption::encrypt_password,
    helper::{json_to_string_vec, unordered_vec_equals},
  },
  model::rbac::sql::{PermissionSQLRow, RolePermissionSQLRow, RoleSQLRow},
};

#[derive(Clone, sqlx::FromRow)]
pub struct UserSQLRow {
  pub id: i64,
  pub username: String,
  pub active: i64,
  pub updated_at: i64,
  pub created_at: i64,
}

impl UserSQLRow {
  pub fn is_active(&self) -> bool {
    self.active != 0
  }
}

#[derive(sqlx::FromRow, Default)]
pub struct UserInfoSQLRow {
  pub user_id: i64,
  pub name: Option<String>,
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
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(sqlx::FromRow)]
pub struct UserPasswordSQLRow {
  pub id: i64,
  pub user_id: i64,
  pub active: i64,
  pub encrypted_password: String,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(sqlx::FromRow)]
pub struct UserEmailSQLRow {
  pub id: i64,
  pub user_id: i64,
  pub primary: i64,
  pub verified: i64,
  pub email: String,
  pub updated_at: i64,
  pub created_at: i64,
}

impl UserEmailSQLRow {
  pub fn is_verified(&self) -> bool {
    self.verified != 0
  }
  pub fn is_primary(&self) -> bool {
    self.primary != 0
  }
}

#[derive(sqlx::FromRow)]
pub struct UserPhoneNumberSQLRow {
  pub id: i64,
  pub user_id: i64,
  pub primary: i64,
  pub verified: i64,
  pub phone_number: String,
  pub updated_at: i64,
  pub created_at: i64,
}

impl UserPhoneNumberSQLRow {
  pub fn is_verified(&self) -> bool {
    self.verified != 0
  }
  pub fn is_primary(&self) -> bool {
    self.primary != 0
  }
}

#[derive(sqlx::FromRow)]
pub struct UserOAuth2ProviderSQLRow {
  pub oauth2_provider_id: i64,
  pub user_id: i64,
  pub uri: String,
  pub name: String,
  pub email: String,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(sqlx::FromRow)]
pub struct UserClientSQLRow {
  pub client_id: String,
  pub user_id: i64,
  pub allowed_scopes: String,
  pub updated_at: i64,
  pub created_at: i64,
}

pub async fn get_user_by_id(pool: &sqlx::AnyPool, id: i64) -> sqlx::Result<Option<UserSQLRow>> {
  sqlx::query_as(
    r#"SELECT u.*
    FROM users u
    WHERE u.id = $1
    LIMIT 1;"#,
  )
  .bind(id)
  .fetch_optional(pool)
  .await
}

pub async fn get_user_by_username_or_primary_email(
  pool: &sqlx::AnyPool,
  username_or_email: &str,
) -> sqlx::Result<Option<UserSQLRow>> {
  sqlx::query_as(
    r#"SELECT u.*
    FROM users u
    LEFT JOIN user_emails ue ON ue.user_id = u.id
    WHERE u.username = $1 OR (ue.email = $1 AND ue."primary" != 0)
    LIMIT 1;"#,
  )
  .bind(username_or_email)
  .fetch_optional(pool)
  .await
}

pub async fn get_user_by_username(
  pool: &sqlx::AnyPool,
  username: &str,
) -> sqlx::Result<Option<UserSQLRow>> {
  sqlx::query_as(
    r#"SELECT u.*
    FROM users u
    WHERE u.username = $1
    LIMIT 1;"#,
  )
  .bind(username)
  .fetch_optional(pool)
  .await
}

pub async fn get_user_active_password_by_user_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Option<UserPasswordSQLRow>> {
  sqlx::query_as(
    r#"SELECT up.*
    FROM user_passwords up
    WHERE up.active != 0 AND up.user_id = $1 
    LIMIT 1;"#,
  )
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

#[derive(Default)]
pub struct UserInfoUpdate {
  pub name: Option<String>,
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

async fn create_user_internal(
  transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
  username: &str,
  user_info: UserInfoUpdate,
) -> sqlx::Result<(UserSQLRow, UserInfoSQLRow)> {
  let user: UserSQLRow =
    sqlx::query_as(r#"INSERT INTO users ("username", "active") VALUES ($1, $2) RETURNING *;"#)
      .bind(&username)
      .bind(true)
      .fetch_one(&mut **transaction)
      .await?;

  let user_info = sqlx::query_as(r#"INSERT INTO user_infos 
      ("user_id", "name", "given_name", "family_name", "middle_name", "nickname", "profile_picture", "website", "gender", "birthdate", "zone_info", "locale", "address")
      VALUES 
      ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
      RETURNING *;"#)
    .bind(user.id)
    .bind(user_info.name)
    .bind(user_info.given_name)
    .bind(user_info.family_name)
    .bind(user_info.middle_name)
    .bind(user_info.nickname.unwrap_or(username.to_owned()))
    .bind(user_info.profile_picture)
    .bind(user_info.website)
    .bind(user_info.gender)
    .bind(user_info.birthdate)
    .bind(user_info.zone_info)
    .bind(user_info.locale)
    .bind(user_info.address)
    .fetch_one(&mut **transaction)
    .await?;

  Ok((user, user_info))
}

pub async fn create_user_with_password(
  pool: &sqlx::AnyPool,
  app_config: &AppConfig,
  username: &str,
  password: &str,
) -> sqlx::Result<UserSQLRow> {
  let encrypted_password = match encrypt_password(app_config, &password) {
    Ok(encrypted_password) => encrypted_password,
    Err(e) => {
      return Err(sqlx::Error::Encode(
        format!("Failed to encrypt password: {}", e).into(),
      ));
    }
  };
  let username_owned = username.to_owned();
  run_transaction(pool, |transaction| {
    Box::pin(async move {
      let (user, _user_info) =
        create_user_internal(transaction, &username_owned, UserInfoUpdate::default()).await?;

      sqlx::query(
        r#"INSERT INTO user_passwords ("user_id", "encrypted_password") VALUES ($1, $2);"#,
      )
      .bind(user.id)
      .bind(encrypted_password)
      .execute(&mut **transaction)
      .await?;

      Ok(user)
    })
  })
  .await
}

pub async fn get_user_oauth2_providers(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Vec<UserOAuth2ProviderSQLRow>> {
  sqlx::query_as(
    r#"SELECT p.uri, p.description, uop.*
    FROM user_oauth2_providers uop
      JOIN oauth2_providers p ON uop.oauth2_provider_id = p.id
    WHERE uop.user_id = $1 AND p.active != 0;"#,
  )
  .bind(user_id)
  .fetch_all(pool)
  .await
}

pub async fn get_user_info_by_user_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Option<UserInfoSQLRow>> {
  sqlx::query_as(
    r#"SELECT ui.*
    FROM user_infos ui
    JOIN users u on ui.user_id = u.id
    WHERE ui.user_id = $1
    LIMIT 1;"#,
  )
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

pub async fn get_user_emails_by_user_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Vec<UserEmailSQLRow>> {
  sqlx::query_as(
    r#"SELECT ue.*
    FROM user_emails ue
    JOIN users u ON u.id = ue.user_id
    WHERE ue.user_id = $1;"#,
  )
  .bind(user_id)
  .fetch_all(pool)
  .await
}

pub async fn get_user_primary_email(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Option<UserEmailSQLRow>> {
  sqlx::query_as(
    r#"SELECT ue.*
    FROM user_emails ue
    JOIN users u ON u.id = ue.user_id
    WHERE ue.user_id = $1 AND ue.primary != 0;"#,
  )
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

pub async fn get_user_phone_numbers_by_user_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Vec<UserPhoneNumberSQLRow>> {
  sqlx::query_as(
    r#"SELECT upn.*
    FROM user_phone_numbers upn
    JOIN users u ON u.id = upn.user_id
    WHERE upn.user_id = $1;"#,
  )
  .bind(user_id)
  .fetch_all(pool)
  .await
}

pub async fn get_user_primary_phone_number(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Option<UserPhoneNumberSQLRow>> {
  sqlx::query_as(
    r#"SELECT upn.*
    FROM user_phone_numbers upn
    JOIN users u ON u.id = upn.user_id
    WHERE upn.user_id = $1 AND upn.primary != 0;"#,
  )
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

pub async fn get_user_roles_by_user_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Vec<RoleSQLRow>> {
  sqlx::query_as(
    r#"SELECT r.*
    FROM user_roles ur
    JOIN roles r ON r.id = ur.role_id
    WHERE ur.user_id = $1;"#,
  )
  .bind(user_id)
  .fetch_all(pool)
  .await
}

// returns map of role_id -> Vec<PermissionSQLRow>
pub async fn get_user_role_permissions_by_user_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<HashMap<i64, Vec<PermissionSQLRow>>> {
  let roles_permissions: Vec<RolePermissionSQLRow> = sqlx::query_as(
    r#"SELECT p.*, rp.role_id
    FROM user_roles ur
    JOIN roles_permissions rp on rp.role_id = ur.role_id
    JOIN permissions p ON p.id = rp.permission_id
    WHERE ur.user_id = $1;"#,
  )
  .bind(user_id)
  .fetch_all(pool)
  .await?;

  let mut permissions = HashMap::default();

  for roles_permission in roles_permissions {
    permissions
      .entry(roles_permission.role_id)
      .or_insert_with(Vec::new)
      .push(roles_permission.into());
  }

  Ok(permissions)
}

pub async fn get_user_client_by_client_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
  client_id: &str,
) -> sqlx::Result<Option<UserClientSQLRow>> {
  get_user_client_by_client_id_internal(pool, user_id, client_id).await
}

async fn get_user_client_by_client_id_internal<'e, E>(
  executor: E,
  user_id: i64,
  client_id: &str,
) -> sqlx::Result<Option<UserClientSQLRow>>
where
  E: 'e + sqlx::Executor<'e, Database = sqlx::Any>,
{
  sqlx::query_as(
    r#"SELECT uc.* 
      FROM "user_clients" uc
      WHERE uc.user_id = $1 AND uc.client_id = $2;"#,
  )
  .bind(user_id)
  .bind(client_id)
  .fetch_optional(executor)
  .await
}

async fn update_user_client_internal<'e, E>(
  executor: E,
  user_id: i64,
  client_id: &str,
  allowed_scopes: &str,
) -> sqlx::Result<UserClientSQLRow>
where
  E: 'e + sqlx::Executor<'e, Database = sqlx::Any>,
{
  sqlx::query_as(
    r#"UPDATE user_clients 
            SET 
              allowed_scopes = $1,
              updated_at = $2
            WHERE user_id = $3 AND client_id = $4
            RETURNING *;"#,
  )
  .bind(allowed_scopes)
  .bind(chrono::Utc::now().timestamp())
  .bind(user_id)
  .bind(client_id)
  .fetch_one(executor)
  .await
}

async fn create_user_client_internal<'e, E>(
  executor: E,
  user_id: i64,
  client_id: &str,
  allowed_scopes: &str,
) -> sqlx::Result<UserClientSQLRow>
where
  E: 'e + sqlx::Executor<'e, Database = sqlx::Any>,
{
  sqlx::query_as(
    r#"INSERT INTO "user_clients" 
      ("user_id", "client_id", "allowed_scopes") 
      VALUES ($1, $2, $3) 
      RETURNING *;"#,
  )
  .bind(user_id)
  .bind(client_id)
  .bind(allowed_scopes)
  .fetch_one(executor)
  .await
}

pub async fn upsert_user_client(
  pool: &sqlx::AnyPool,
  user_id: i64,
  client_id: String,
  allowed_scopes_json: String,
) -> sqlx::Result<UserClientSQLRow> {
  let allowed_scopes = json_to_string_vec(&allowed_scopes_json);
  run_transaction(pool, |transaction| {
    Box::pin(async move {
      match get_user_client_by_client_id_internal(&mut **transaction, user_id, &client_id).await? {
        Some(user_client_sql_row) => {
          if unordered_vec_equals(
            &json_to_string_vec(&user_client_sql_row.allowed_scopes),
            &allowed_scopes,
          ) {
            Ok(user_client_sql_row)
          } else {
            update_user_client_internal(
              &mut **transaction,
              user_id,
              &client_id,
              &allowed_scopes_json,
            )
            .await
          }
        }
        None => {
          create_user_client_internal(
            &mut **transaction,
            user_id,
            &client_id,
            &allowed_scopes_json,
          )
          .await
        }
      }
    })
  })
  .await
}
