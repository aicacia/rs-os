use os_db::pool::run_transaction;

use crate::core::{config::dynamic_app_config::DynamicAppConfig, encryption::encrypt_password};

#[derive(sqlx::FromRow)]
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
  username: String,
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
    .bind(user_info.nickname.unwrap_or(username))
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
  dynamic_app_config: &DynamicAppConfig,
  username: String,
  password: String,
) -> sqlx::Result<UserSQLRow> {
  let encrypted_password = match encrypt_password(dynamic_app_config, &password) {
    Ok(encrypted_password) => encrypted_password,
    Err(e) => {
      return Err(sqlx::Error::Encode(
        format!("Failed to encrypt password: {}", e).into(),
      ));
    }
  };
  run_transaction(pool, |transaction| {
    Box::pin(async move {
      let (user, _user_info) =
        create_user_internal(transaction, username, UserInfoUpdate::default()).await?;

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
    r#"SELECT p.uri, p.name, uop.*
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
