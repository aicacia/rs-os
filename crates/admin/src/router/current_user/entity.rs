use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use os_model::entities::{
  roles, user_emails,
  user_infos::{self, UserInfoUpdate},
  user_o_auth2_providers, user_phone_numbers, users,
};

#[derive(Serialize, ToSchema, Default)]
pub struct CurrentUser {
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

impl From<users::Model> for CurrentUser {
  fn from(user_model: users::Model) -> Self {
    Self {
      active: user_model.is_active(),
      id: user_model.id,
      username: user_model.username,
      updated_at: DateTime::<Utc>::from_timestamp(user_model.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_model.created_at, 0).unwrap_or_default(),
      ..Self::default()
    }
  }
}

#[derive(Serialize, ToSchema, Default)]
pub struct UserRole {
  pub uri: String,
  pub permissions: Vec<String>,
}

impl From<roles::Model> for UserRole {
  fn from(role_model: roles::Model) -> Self {
    Self {
      uri: role_model.uri,
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

impl From<user_infos::Model> for UserInfo {
  fn from(user_info_model: user_infos::Model) -> Self {
    let name = if let Some(given_name) = &user_info_model.given_name {
      if let Some(family_name) = &user_info_model.family_name {
        Some(format!("{} {}", given_name, family_name))
      } else {
        Some(given_name.clone())
      }
    } else {
      user_info_model.family_name.clone()
    };

    Self {
      name,
      given_name: user_info_model.given_name,
      family_name: user_info_model.family_name,
      middle_name: user_info_model.middle_name,
      nickname: user_info_model.nickname,
      profile_picture: user_info_model.profile_picture,
      website: user_info_model.website,
      gender: user_info_model.gender,
      birthdate: user_info_model.birthdate,
      zone_info: user_info_model.zone_info,
      locale: user_info_model.locale,
      address: user_info_model.address,
      updated_at: DateTime::<Utc>::from_timestamp(user_info_model.updated_at, 0)
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

impl From<user_emails::Model> for UserEmail {
  fn from(user_email_model: user_emails::Model) -> Self {
    Self {
      primary: user_email_model.is_primary(),
      verified: user_email_model.is_verified(),

      id: user_email_model.id,
      email: user_email_model.email,
      updated_at: DateTime::<Utc>::from_timestamp(user_email_model.updated_at, 0)
        .unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_email_model.created_at, 0)
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

impl From<user_phone_numbers::Model> for UserPhoneNumber {
  fn from(user_phone_number_model: user_phone_numbers::Model) -> Self {
    Self {
      primary: user_phone_number_model.is_primary(),
      verified: user_phone_number_model.is_verified(),

      id: user_phone_number_model.id,
      phone_number: user_phone_number_model.phone_number,
      updated_at: DateTime::<Utc>::from_timestamp(user_phone_number_model.updated_at, 0)
        .unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_phone_number_model.created_at, 0)
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

// Note: This From implementation should not be used directly.
// Use the conversion logic in middleware/user_authorization.rs that joins with provider info.
impl From<user_o_auth2_providers::Model> for UserOAuth2Provider {
  fn from(user_oauth2_provider_model: user_o_auth2_providers::Model) -> Self {
    Self {
      id: user_oauth2_provider_model.o_auth2_provider_id,
      uri: String::new(),  // Must be populated from o_auth2_providers table
      name: String::new(), // Must be populated from o_auth2_providers table
      email: Some(user_oauth2_provider_model.email),
      updated_at: DateTime::<Utc>::from_timestamp(user_oauth2_provider_model.updated_at, 0)
        .unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(user_oauth2_provider_model.created_at, 0)
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

#[derive(Validate, Deserialize, ToSchema)]
pub struct UpdateUsernameRequest {
  #[validate(length(min = 1))]
  pub username: String,
}

#[derive(Validate, Deserialize, ToSchema)]
pub struct UpdateUserInfoRequest {
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

impl From<UpdateUserInfoRequest> for UserInfoUpdate {
  fn from(req: UpdateUserInfoRequest) -> Self {
    UserInfoUpdate {
      given_name: req.given_name,
      family_name: req.family_name,
      middle_name: req.middle_name,
      nickname: req.nickname,
      profile_picture: req.profile_picture,
      website: req.website,
      gender: req.gender,
      birthdate: req.birthdate,
      zone_info: req.zone_info,
      locale: req.locale,
      address: req.address,
    }
  }
}
