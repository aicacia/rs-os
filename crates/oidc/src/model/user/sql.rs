use hashbrown::HashMap;

use os_db::pool::run_transaction;

use crate::{
  core::{
    config::app_config::AppConfig,
    encryption::encrypt_password,
    helper::{json_to_string_vec, unordered_vec_equals},
  },
  model::rbac::sql::{PermissionSQLRow, RolePermissionSQLRow, RoleSQLRow},
  router::common::permissions::Permission,
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

pub async fn list_users(pool: &sqlx::AnyPool) -> sqlx::Result<Vec<UserSQLRow>> {
  sqlx::query_as::<_, UserSQLRow>(
    r#"SELECT u.* FROM users u WHERE u.active != 0 ORDER BY u.created_at DESC;"#,
  )
  .fetch_all(pool)
  .await
}

pub async fn create_user(pool: &sqlx::AnyPool, username: &str) -> sqlx::Result<UserSQLRow> {
  sqlx::query_as::<_, UserSQLRow>(
    r#"INSERT INTO users ("username", "active") VALUES ($1, $2) RETURNING *;"#,
  )
  .bind(username)
  .bind(1)
  .fetch_one(pool)
  .await
}

pub async fn update_user(
  pool: &sqlx::AnyPool,
  user_id: i64,
  username: Option<&str>,
) -> sqlx::Result<Option<UserSQLRow>> {
  if let Some(username) = username {
    sqlx::query_as::<_, UserSQLRow>(
      r#"UPDATE users
        SET
          username = $1,
          updated_at = $2
        WHERE id = $3 AND active != 0
        RETURNING *;"#,
    )
    .bind(username)
    .bind(chrono::Utc::now().timestamp())
    .bind(user_id)
    .fetch_optional(pool)
    .await
  } else {
    get_user_by_id(pool, user_id).await
  }
}

pub async fn delete_user(pool: &sqlx::AnyPool, user_id: i64) -> sqlx::Result<Option<UserSQLRow>> {
  sqlx::query_as::<_, UserSQLRow>(
    r#"UPDATE users
      SET
        active = 0,
        updated_at = $1
      WHERE id = $2 AND active != 0
      RETURNING *;"#,
  )
  .bind(chrono::Utc::now().timestamp())
  .bind(user_id)
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
      ("user_id", "given_name", "family_name", "middle_name", "nickname", "profile_picture", "website", "gender", "birthdate", "zone_info", "locale", "address")
      VALUES 
      ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
      RETURNING *;"#)
    .bind(user.id)
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

pub async fn get_user_email_by_id(
  pool: &sqlx::AnyPool,
  email_id: i64,
) -> sqlx::Result<Option<UserEmailSQLRow>> {
  sqlx::query_as(
    r#"SELECT ue.*
    FROM user_emails ue
    WHERE ue.id = $1
    LIMIT 1;"#,
  )
  .bind(email_id)
  .fetch_optional(pool)
  .await
}

pub async fn create_user_email(
  pool: &sqlx::AnyPool,
  user_id: i64,
  email: &str,
) -> sqlx::Result<UserEmailSQLRow> {
  sqlx::query_as(
    r#"INSERT INTO user_emails ("user_id", "email", "verified", "primary") 
       VALUES ($1, $2, $3, $4) 
       RETURNING *;"#,
  )
  .bind(user_id)
  .bind(email)
  .bind(0)
  .bind(0)
  .fetch_one(pool)
  .await
}

pub async fn update_user_email_primary(
  pool: &sqlx::AnyPool,
  user_id: i64,
  email_id: i64,
) -> sqlx::Result<Option<UserEmailSQLRow>> {
  run_transaction(pool, |transaction| {
    Box::pin(async move {
      // First, unset all other primary emails for this user
      sqlx::query(
        r#"UPDATE user_emails
           SET "primary" = 0, updated_at = $1
           WHERE user_id = $2 AND "primary" != 0;"#,
      )
      .bind(chrono::Utc::now().timestamp())
      .bind(user_id)
      .execute(&mut **transaction)
      .await?;

      // Then set the specified email as primary
      let email: Option<UserEmailSQLRow> = sqlx::query_as(
        r#"UPDATE user_emails
           SET "primary" = 1, updated_at = $1
           WHERE id = $2 AND user_id = $3
           RETURNING *;"#,
      )
      .bind(chrono::Utc::now().timestamp())
      .bind(email_id)
      .bind(user_id)
      .fetch_optional(&mut **transaction)
      .await?;

      Ok(email)
    })
  })
  .await
}

pub async fn verify_user_email(
  pool: &sqlx::AnyPool,
  user_id: i64,
  email_id: i64,
) -> sqlx::Result<Option<UserEmailSQLRow>> {
  sqlx::query_as(
    r#"UPDATE user_emails
       SET verified = 1, updated_at = $1
       WHERE id = $2 AND user_id = $3
       RETURNING *;"#,
  )
  .bind(chrono::Utc::now().timestamp())
  .bind(email_id)
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

pub async fn delete_user_email(
  pool: &sqlx::AnyPool,
  user_id: i64,
  email_id: i64,
) -> sqlx::Result<Option<UserEmailSQLRow>> {
  sqlx::query_as(
    r#"DELETE FROM user_emails
       WHERE id = $1 AND user_id = $2
       RETURNING *;"#,
  )
  .bind(email_id)
  .bind(user_id)
  .fetch_optional(pool)
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
    WHERE ue.user_id = $1 AND ue."primary" != 0;"#,
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

pub async fn get_user_phone_number_by_id(
  pool: &sqlx::AnyPool,
  phone_id: i64,
) -> sqlx::Result<Option<UserPhoneNumberSQLRow>> {
  sqlx::query_as(
    r#"SELECT upn.*
    FROM user_phone_numbers upn
    WHERE upn.id = $1
    LIMIT 1;"#,
  )
  .bind(phone_id)
  .fetch_optional(pool)
  .await
}

pub async fn create_user_phone_number(
  pool: &sqlx::AnyPool,
  user_id: i64,
  phone_number: &str,
) -> sqlx::Result<UserPhoneNumberSQLRow> {
  sqlx::query_as(
    r#"INSERT INTO user_phone_numbers ("user_id", "phone_number", "verified", "primary") 
       VALUES ($1, $2, $3, $4) 
       RETURNING *;"#,
  )
  .bind(user_id)
  .bind(phone_number)
  .bind(0)
  .bind(0)
  .fetch_one(pool)
  .await
}

pub async fn update_user_phone_number_primary(
  pool: &sqlx::AnyPool,
  user_id: i64,
  phone_id: i64,
) -> sqlx::Result<Option<UserPhoneNumberSQLRow>> {
  run_transaction(pool, |transaction| {
    Box::pin(async move {
      // First, unset all other primary phone numbers for this user
      sqlx::query(
        r#"UPDATE user_phone_numbers
           SET "primary" = 0, updated_at = $1
           WHERE user_id = $2 AND "primary" != 0;"#,
      )
      .bind(chrono::Utc::now().timestamp())
      .bind(user_id)
      .execute(&mut **transaction)
      .await?;

      // Then set the specified phone number as primary
      let phone: Option<UserPhoneNumberSQLRow> = sqlx::query_as(
        r#"UPDATE user_phone_numbers
           SET "primary" = 1, updated_at = $1
           WHERE id = $2 AND user_id = $3
           RETURNING *;"#,
      )
      .bind(chrono::Utc::now().timestamp())
      .bind(phone_id)
      .bind(user_id)
      .fetch_optional(&mut **transaction)
      .await?;

      Ok(phone)
    })
  })
  .await
}

pub async fn verify_user_phone_number(
  pool: &sqlx::AnyPool,
  user_id: i64,
  phone_id: i64,
) -> sqlx::Result<Option<UserPhoneNumberSQLRow>> {
  sqlx::query_as(
    r#"UPDATE user_phone_numbers
       SET verified = 1, updated_at = $1
       WHERE id = $2 AND user_id = $3
       RETURNING *;"#,
  )
  .bind(chrono::Utc::now().timestamp())
  .bind(phone_id)
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

pub async fn delete_user_phone_number(
  pool: &sqlx::AnyPool,
  user_id: i64,
  phone_id: i64,
) -> sqlx::Result<Option<UserPhoneNumberSQLRow>> {
  sqlx::query_as(
    r#"DELETE FROM user_phone_numbers
       WHERE id = $1 AND user_id = $2
       RETURNING *;"#,
  )
  .bind(phone_id)
  .bind(user_id)
  .fetch_optional(pool)
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
    WHERE upn.user_id = $1 AND upn."primary" != 0;"#,
  )
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

pub async fn update_user_username(
  pool: &sqlx::AnyPool,
  user_id: i64,
  username: String,
) -> sqlx::Result<UserSQLRow> {
  sqlx::query_as(
    r#"UPDATE users
      SET
        username = $1,
        updated_at = $2
      WHERE id = $3
      RETURNING *;"#,
  )
  .bind(username)
  .bind(chrono::Utc::now().timestamp())
  .bind(user_id)
  .fetch_one(pool)
  .await
}

pub async fn get_user_oauth2_provider_by_id(
  pool: &sqlx::AnyPool,
  user_id: i64,
  provider_id: i64,
) -> sqlx::Result<Option<UserOAuth2ProviderSQLRow>> {
  sqlx::query_as(
    r#"SELECT p.uri, p.description as name, uop.*
    FROM user_oauth2_providers uop
      JOIN oauth2_providers p ON uop.oauth2_provider_id = p.id
    WHERE uop.user_id = $1 AND uop.oauth2_provider_id = $2 AND p.active != 0
    LIMIT 1;"#,
  )
  .bind(user_id)
  .bind(provider_id)
  .fetch_optional(pool)
  .await
}

pub async fn link_user_oauth2_provider(
  pool: &sqlx::AnyPool,
  user_id: i64,
  oauth2_provider_id: i64,
  name: &str,
  email: &str,
) -> sqlx::Result<UserOAuth2ProviderSQLRow> {
  sqlx::query_as(
    r#"INSERT INTO user_oauth2_providers ("user_id", "oauth2_provider_id", "name", "email") 
       VALUES ($1, $2, $3, $4) 
       RETURNING (SELECT uri FROM oauth2_providers WHERE id = $2) as uri, $3 as name, *;"#,
  )
  .bind(user_id)
  .bind(oauth2_provider_id)
  .bind(name)
  .bind(email)
  .fetch_one(pool)
  .await
}

pub async fn delete_user_oauth2_provider(
  pool: &sqlx::AnyPool,
  user_id: i64,
  oauth2_provider_id: i64,
) -> sqlx::Result<Option<UserOAuth2ProviderSQLRow>> {
  sqlx::query_as(
    r#"DELETE FROM user_oauth2_providers
       WHERE user_id = $1 AND oauth2_provider_id = $2
       RETURNING 
         (SELECT uri FROM oauth2_providers WHERE id = $2) as uri,
         (SELECT description FROM oauth2_providers WHERE id = $2) as name,
         *;"#,
  )
  .bind(user_id)
  .bind(oauth2_provider_id)
  .fetch_optional(pool)
  .await
}

pub async fn update_user_info(
  pool: &sqlx::AnyPool,
  user_id: i64,
  user_info: UserInfoUpdate,
) -> sqlx::Result<UserInfoSQLRow> {
  sqlx::query_as(
    r#"UPDATE user_infos
      SET
        given_name = COALESCE($1, given_name),
        family_name = COALESCE($2, family_name),
        middle_name = COALESCE($3, middle_name),
        nickname = COALESCE($4, nickname),
        profile_picture = COALESCE($5, profile_picture),
        website = COALESCE($6, website),
        gender = COALESCE($7, gender),
        birthdate = COALESCE($8, birthdate),
        zone_info = COALESCE($9, zone_info),
        locale = COALESCE($10, locale),
        address = COALESCE($11, address),
        updated_at = $12
      WHERE user_id = $13
      RETURNING *;"#,
  )
  .bind(user_info.given_name)
  .bind(user_info.family_name)
  .bind(user_info.middle_name)
  .bind(user_info.nickname)
  .bind(user_info.profile_picture)
  .bind(user_info.website)
  .bind(user_info.gender)
  .bind(user_info.birthdate)
  .bind(user_info.zone_info)
  .bind(user_info.locale)
  .bind(user_info.address)
  .bind(chrono::Utc::now().timestamp())
  .bind(user_id)
  .fetch_one(pool)
  .await
}

pub async fn update_user_password(
  pool: &sqlx::AnyPool,
  app_config: &AppConfig,
  user_id: i64,
  password: &str,
) -> sqlx::Result<()> {
  let encrypted_password = match encrypt_password(app_config, &password) {
    Ok(encrypted_password) => encrypted_password,
    Err(e) => {
      return Err(sqlx::Error::Encode(
        format!("Failed to encrypt password: {}", e).into(),
      ));
    }
  };

  run_transaction(pool, |transaction| {
    let encrypted_password = encrypted_password.clone();
    Box::pin(async move {
      // deactivate any existing active passwords for this user
      sqlx::query(r#"UPDATE user_passwords SET active = 0 WHERE user_id = $1 AND active != 0;"#)
        .bind(user_id)
        .execute(&mut **transaction)
        .await?;

      // insert new active password row
      sqlx::query(
        r#"INSERT INTO user_passwords ("user_id", "encrypted_password", "active") VALUES ($1, $2, $3);"#,
      )
      .bind(user_id)
      .bind(encrypted_password)
      .bind(1)
      .execute(&mut **transaction)
      .await?;

      Ok(())
    })
  })
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

pub async fn get_user_roles(pool: &sqlx::AnyPool, user_id: i64) -> sqlx::Result<Vec<RoleSQLRow>> {
  get_user_roles_by_user_id(pool, user_id).await
}

pub async fn get_user_permissions(
  pool: &sqlx::AnyPool,
  user_id: i64,
) -> sqlx::Result<Vec<Permission>> {
  let role_permissions = get_user_role_permissions_by_user_id(pool, user_id).await?;

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
  pool: &sqlx::AnyPool,
  user_id: i64,
  role_id: i64,
) -> sqlx::Result<RoleSQLRow> {
  // First insert the user_role relationship
  sqlx::query(
    r#"INSERT INTO user_roles ("user_id", "role_id") VALUES ($1, $2) 
       ON CONFLICT (user_id, role_id) DO NOTHING;"#,
  )
  .bind(user_id)
  .bind(role_id)
  .execute(pool)
  .await?;

  // Then fetch and return the role
  sqlx::query_as(r#"SELECT r.* FROM roles r WHERE r.id = $1 LIMIT 1;"#)
    .bind(role_id)
    .fetch_one(pool)
    .await
}

pub async fn remove_user_role(
  pool: &sqlx::AnyPool,
  user_id: i64,
  role_id: i64,
) -> sqlx::Result<Option<RoleSQLRow>> {
  // First check if the role exists
  let role: Option<RoleSQLRow> = sqlx::query_as(
    r#"SELECT r.* FROM roles r 
       JOIN user_roles ur ON ur.role_id = r.id
       WHERE ur.user_id = $1 AND ur.role_id = $2 
       LIMIT 1;"#,
  )
  .bind(user_id)
  .bind(role_id)
  .fetch_optional(pool)
  .await?;

  if role.is_some() {
    // Delete the user_role relationship
    sqlx::query(r#"DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2;"#)
      .bind(user_id)
      .bind(role_id)
      .execute(pool)
      .await?;
  }

  Ok(role)
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
