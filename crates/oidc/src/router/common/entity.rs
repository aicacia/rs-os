use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
pub use os_api::claims::{BasicClaims, Claims};
use os_oidc_model::entities::users;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Token {
  pub access_token: String,
  pub token_type: String,
  pub issued_token_type: String,
  pub issued_at: DateTime<Utc>,
  pub expires_in: i64,
  pub scope: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub refresh_token: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub refresh_token_expires_in: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id_token: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub password_reset_required: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, ToSchema)]
pub struct AuthorizationCodeClaims {
  #[serde(flatten)]
  pub basic_claims: BasicClaims,
  pub code_challenge: String,
  pub code_challenge_method: String,
}

impl Claims for AuthorizationCodeClaims {
  fn r#type(&self) -> &str {
    &self.basic_claims.r#type
  }
  fn exp(&self) -> i64 {
    self.basic_claims.exp
  }
  fn iat(&self) -> i64 {
    self.basic_claims.iat
  }
  fn nbf(&self) -> i64 {
    self.basic_claims.nbf
  }
  fn iss(&self) -> &str {
    &self.basic_claims.iss
  }
  fn user(&self) -> i64 {
    self.basic_claims.user
  }
  fn client(&self) -> &str {
    &self.basic_claims.client
  }
  fn aud(&self) -> &[String] {
    &self.basic_claims.aud
  }
  fn sub(&self) -> &str {
    &self.basic_claims.sub
  }
  fn scope(&self) -> &str {
    &self.basic_claims.scope
  }
}

#[derive(Serialize, Deserialize, Default, Clone, ToSchema)]
pub struct OpenIdProfile {
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
  pub preferred_username: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub profile_picture: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub website: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email_verified: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub gender: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub birthdate: Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub zone_info: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub locale: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub phone: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub phone_verified: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub address: Option<String>,
}

impl From<os_oidc_model::entities::user_infos::Model> for OpenIdProfile {
  fn from(user_info_model: os_oidc_model::entities::user_infos::Model) -> Self {
    let name: Option<String> = if let Some(given_name) = &user_info_model.given_name {
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
      preferred_username: None,
      profile_picture: user_info_model.profile_picture,
      website: user_info_model.website,
      gender: user_info_model.gender,
      birthdate: user_info_model
        .birthdate
        .and_then(|birthdate| DateTime::<Utc>::from_timestamp(birthdate, 0)),
      zone_info: user_info_model.zone_info,
      locale: user_info_model.locale,
      address: user_info_model.address,
      email: None,
      email_verified: None,
      phone: None,
      phone_verified: None,
    }
  }
}

#[derive(Serialize, Deserialize, Default, Clone, ToSchema)]
pub struct OpenIdClaims {
  #[serde(flatten)]
  pub basic_claims: BasicClaims,
  #[serde(flatten)]
  pub profile: OpenIdProfile,
  pub username: String,
}

impl Claims for OpenIdClaims {
  fn r#type(&self) -> &str {
    &self.basic_claims.r#type
  }
  fn exp(&self) -> i64 {
    self.basic_claims.exp
  }
  fn iat(&self) -> i64 {
    self.basic_claims.iat
  }
  fn nbf(&self) -> i64 {
    self.basic_claims.nbf
  }
  fn iss(&self) -> &str {
    &self.basic_claims.iss
  }
  fn user(&self) -> i64 {
    self.basic_claims.user
  }
  fn client(&self) -> &str {
    &self.basic_claims.client
  }
  fn aud(&self) -> &[String] {
    &self.basic_claims.aud
  }
  fn sub(&self) -> &str {
    &self.basic_claims.sub
  }
  fn scope(&self) -> &str {
    &self.basic_claims.scope
  }
}

pub trait EncodeClaims: Claims {
  fn encode(
    &self,
    issuer: &str,
    jwk: &jsonwebtoken::jwk::Jwk,
    encoding_key: &jsonwebtoken::EncodingKey,
  ) -> Result<String, jsonwebtoken::errors::Error> {
    use std::str::FromStr;
    let algorithm = match jwk.common.key_algorithm {
      Some(key_algorithm) => {
        match jsonwebtoken::Algorithm::from_str(key_algorithm.to_string().as_str()) {
          Ok(algorithm) => algorithm,
          Err(e) => {
            log::error!("failed to convert key algorithm into string: {}", e);
            return Err(jsonwebtoken::errors::Error::from(
              jsonwebtoken::errors::ErrorKind::InvalidAlgorithmName,
            ));
          }
        }
      }
      None => {
        return Err(jsonwebtoken::errors::Error::from(
          jsonwebtoken::errors::ErrorKind::InvalidAlgorithmName,
        ));
      }
    };
    let mut header = jsonwebtoken::Header::new(algorithm);
    match &jwk.common.key_id {
      Some(kid) => {
        header.kid = Some(kid.clone());
        header.jku = Some(format!("{}/.well-known/jwks.json", issuer))
      }
      None => {
        log::error!("failed to get JWK ID");
        return Err(jsonwebtoken::errors::Error::from(
          jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
        ));
      }
    }
    jsonwebtoken::encode(&header, self, encoding_key)
  }
}

impl EncodeClaims for BasicClaims {}

impl EncodeClaims for AuthorizationCodeClaims {}

impl EncodeClaims for OpenIdClaims {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  #[serde(rename = "*")]
  All,

  #[serde(rename = "client:*")]
  ClientAll,
  #[serde(rename = "client:read")]
  ClientRead,
  #[serde(rename = "client:create")]
  ClientCreate,
  #[serde(rename = "client:update")]
  ClientUpdate,
  #[serde(rename = "client:delete")]
  ClientDelete,

  #[serde(rename = "user:*")]
  UserAll,
  #[serde(rename = "user:read")]
  UserRead,
  #[serde(rename = "user:create")]
  UserCreate,
  #[serde(rename = "user:update")]
  UserUpdate,
  #[serde(rename = "user:delete")]
  UserDelete,
}

impl Permission {
  pub fn as_str(&self) -> &'static str {
    match self {
      Permission::All => "*",
      Permission::ClientAll => "client:*",
      Permission::ClientRead => "client:read",
      Permission::ClientCreate => "client:create",
      Permission::ClientUpdate => "client:update",
      Permission::ClientDelete => "client:delete",
      Permission::UserAll => "user:*",
      Permission::UserRead => "user:read",
      Permission::UserCreate => "user:create",
      Permission::UserUpdate => "user:update",
      Permission::UserDelete => "user:delete",
    }
  }

  pub fn all() -> Vec<Permission> {
    vec![
      Permission::All,
      Permission::ClientAll,
      Permission::ClientRead,
      Permission::ClientCreate,
      Permission::ClientUpdate,
      Permission::ClientDelete,
      Permission::UserAll,
      Permission::UserRead,
      Permission::UserCreate,
      Permission::UserUpdate,
      Permission::UserDelete,
    ]
  }
}

impl std::fmt::Display for Permission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl FromStr for Permission {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "*" => Ok(Permission::All),
      "client:*" => Ok(Permission::ClientAll),
      "client:read" => Ok(Permission::ClientRead),
      // Backward compatibility: map write -> update
      "client:write" => Ok(Permission::ClientUpdate),
      "client:create" => Ok(Permission::ClientCreate),
      "client:update" => Ok(Permission::ClientUpdate),
      "client:delete" => Ok(Permission::ClientDelete),
      "user:*" => Ok(Permission::UserAll),
      "user:read" => Ok(Permission::UserRead),
      // Backward compatibility: map write -> update
      "user:write" => Ok(Permission::UserUpdate),
      "user:create" => Ok(Permission::UserCreate),
      "user:update" => Ok(Permission::UserUpdate),
      "user:delete" => Ok(Permission::UserDelete),
      _ => Err(format!("unknown permission: {}", s)),
    }
  }
}

#[derive(Serialize, Deserialize, Default, Clone, ToSchema)]
pub struct UserInfo {
  #[serde(flatten)]
  pub basic_claims: BasicClaims,
  #[serde(flatten)]
  pub profile: OpenIdProfile,
  pub username: String,
  pub roles: HashMap<String, Vec<String>>,
  pub permissions: HashMap<String, Vec<Permission>>,
}

impl From<users::Model> for UserInfo {
  fn from(user_model: users::Model) -> Self {
    Self {
      basic_claims: BasicClaims {
        user: user_model.id,
        sub: format!("urn:os:sub:user:{}", user_model.id),
        ..Default::default()
      },
      profile: OpenIdProfile {
        preferred_username: Some(user_model.username.clone()),
        ..Default::default()
      },
      username: user_model.username,
      roles: HashMap::new(),
      permissions: HashMap::new(),
    }
  }
}
