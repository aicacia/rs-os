use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::model::{
  rbac::sql::RoleSQLRow,
  user::sql::{
    UserEmailSQLRow, UserInfoSQLRow, UserOAuth2ProviderSQLRow, UserPhoneNumberSQLRow, UserSQLRow,
  },
};

#[derive(Serialize, ToSchema, Default)]
pub struct User {
  #[serde(rename = "sub")]
  pub id: i64,
  pub username: String,
  pub active: bool,
  pub roles: Vec<UserRole>,
  pub info: Option<UserInfo>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<UserEmail>,
  pub emails: Vec<UserEmail>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub phone_number: Option<UserPhoneNumber>,
  pub phone_numbers: Vec<UserPhoneNumber>,
  pub oauth2_providers: Vec<UserOAuth2Provider>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<UserSQLRow> for User {
  fn from(user_sql_row: UserSQLRow) -> Self {
    Self {
      active: user_sql_row.is_active(),
      id: user_sql_row.id,
      username: user_sql_row.username,
      updated_at: DateTime::<Utc>::from_timestamp(user_sql_row.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_sql_row.created_at, 0).unwrap_or_default(),
      ..Self::default()
    }
  }
}

#[derive(Serialize, ToSchema, Default)]
pub struct UserRole {
  pub uri: String,
  pub permissions: Vec<String>,
}

impl From<RoleSQLRow> for UserRole {
  fn from(role_sql_row: RoleSQLRow) -> Self {
    Self {
      uri: role_sql_row.uri,
      ..Self::default()
    }
  }
}

#[derive(Serialize, ToSchema, Default)]
pub struct UserInfo {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub given_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub family_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub middle_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nickname: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub profile_picture: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub website: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub gender: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub birthdate: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub zone_info: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub locale: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub address: Option<String>,
  pub updated_at: DateTime<Utc>,
}

impl From<UserInfoSQLRow> for UserInfo {
  fn from(user_info_sql_row: UserInfoSQLRow) -> Self {
    Self {
      name: user_info_sql_row.name,
      given_name: user_info_sql_row.given_name,
      family_name: user_info_sql_row.family_name,
      middle_name: user_info_sql_row.middle_name,
      nickname: user_info_sql_row.nickname,
      profile_picture: user_info_sql_row.profile_picture,
      website: user_info_sql_row.website,
      gender: user_info_sql_row.gender,
      birthdate: user_info_sql_row.birthdate,
      zone_info: user_info_sql_row.zone_info,
      locale: user_info_sql_row.locale,
      address: user_info_sql_row.address,
      updated_at: DateTime::<Utc>::from_timestamp(user_info_sql_row.updated_at, 0)
        .unwrap_or_default(),
    }
  }
}

#[derive(Serialize, ToSchema, Default)]
pub struct UserEmail {
  pub id: i64,
  pub email: String,
  pub primary: bool,
  pub verified: bool,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<UserEmailSQLRow> for UserEmail {
  fn from(user_email_sql_row: UserEmailSQLRow) -> Self {
    Self {
      primary: user_email_sql_row.is_primary(),
      verified: user_email_sql_row.is_verified(),

      id: user_email_sql_row.id,
      email: user_email_sql_row.email,
      updated_at: DateTime::<Utc>::from_timestamp(user_email_sql_row.updated_at, 0)
        .unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_email_sql_row.created_at, 0)
        .unwrap_or_default(),
    }
  }
}

#[derive(Serialize, ToSchema, Default)]
pub struct UserPhoneNumber {
  pub id: i64,
  pub phone_number: String,
  pub primary: bool,
  pub verified: bool,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<UserPhoneNumberSQLRow> for UserPhoneNumber {
  fn from(user_phone_number_sql_row: UserPhoneNumberSQLRow) -> Self {
    Self {
      primary: user_phone_number_sql_row.is_primary(),
      verified: user_phone_number_sql_row.is_verified(),

      id: user_phone_number_sql_row.id,
      phone_number: user_phone_number_sql_row.phone_number,
      updated_at: DateTime::<Utc>::from_timestamp(user_phone_number_sql_row.updated_at, 0)
        .unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_phone_number_sql_row.created_at, 0)
        .unwrap_or_default(),
    }
  }
}

#[derive(Serialize, ToSchema, Default)]
pub struct UserOAuth2Provider {
  pub id: i64,
  pub uri: String,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<UserOAuth2ProviderSQLRow> for UserOAuth2Provider {
  fn from(user_oauth2_provider_sql_row: UserOAuth2ProviderSQLRow) -> Self {
    Self {
      id: user_oauth2_provider_sql_row.oauth2_provider_id,
      uri: user_oauth2_provider_sql_row.uri,
      name: user_oauth2_provider_sql_row.name,
      email: Some(user_oauth2_provider_sql_row.email),
      updated_at: DateTime::<Utc>::from_timestamp(user_oauth2_provider_sql_row.updated_at, 0)
        .unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_oauth2_provider_sql_row.created_at, 0)
        .unwrap_or_default(),
    }
  }
}

#[derive(Validate, Deserialize, ToSchema)]
pub struct UpdateUser {
  #[validate(length(min = 1))]
  pub username: Option<String>,
  pub active: Option<bool>,
}

#[derive(Validate, Deserialize, ToSchema)]
pub struct UpdateUserPassword {
  #[validate(length(min = 6), must_match(other = "password_confirmation"))]
  pub password: String,
  #[validate(length(min = 6))]
  pub password_confirmation: String,
}
