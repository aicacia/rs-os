use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

use crate::{model::user::sql::UserInfoSQLRow, router::common::helper::to_public_jwk};

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
}

pub trait Claims: Serialize + Send + Sync + DeserializeOwned {
  fn r#type(&self) -> &str;
  fn exp(&self) -> i64;
  fn iat(&self) -> i64;
  fn nbf(&self) -> i64;
  fn iss(&self) -> &str;
  fn aud(&self) -> &str;
  fn sub(&self) -> i64;
  fn scope(&self) -> &str;

  fn has_scope(&self, scope: &str) -> bool {
    self.scope().split_whitespace().any(|s| s == scope)
  }

  fn encode(
    &self,
    issuer: &str,
    jwk: &jsonwebtoken::jwk::Jwk,
    encoding_key: &jsonwebtoken::EncodingKey,
  ) -> Result<String, jsonwebtoken::errors::Error> {
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
    header.jwk = Some(to_public_jwk(jwk));
    match &jwk.common.key_id {
      Some(kid) => {
        header.jku = Some(format!("{}/jwks/{}", issuer, kid));
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

#[derive(Debug, Default, Serialize, Deserialize, Clone, ToSchema)]
pub struct BasicClaims {
  pub r#type: String,
  pub exp: i64,
  pub iat: i64,
  pub nbf: i64,
  pub iss: String,
  pub aud: String,
  pub sub: i64,
  pub scope: String,
}

impl Claims for BasicClaims {
  fn r#type(&self) -> &str {
    &self.r#type
  }
  fn exp(&self) -> i64 {
    self.exp
  }
  fn iat(&self) -> i64 {
    self.iat
  }
  fn nbf(&self) -> i64 {
    self.nbf
  }
  fn iss(&self) -> &str {
    &self.iss
  }
  fn aud(&self) -> &str {
    &self.aud
  }
  fn sub(&self) -> i64 {
    self.sub
  }

  fn scope(&self) -> &str {
    &self.scope
  }
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
  fn aud(&self) -> &str {
    &self.basic_claims.aud
  }
  fn sub(&self) -> i64 {
    self.basic_claims.sub
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

impl From<UserInfoSQLRow> for OpenIdProfile {
  fn from(user_info_sql_row: UserInfoSQLRow) -> Self {
    let name: Option<String> = if let Some(given_name) = &user_info_sql_row.given_name {
      if let Some(family_name) = &user_info_sql_row.family_name {
        Some(format!("{} {}", given_name, family_name))
      } else {
        Some(given_name.clone())
      }
    } else if let Some(family_name) = &user_info_sql_row.family_name {
      Some(family_name.clone())
    } else {
      None
    };

    Self {
      name,
      given_name: user_info_sql_row.given_name,
      family_name: user_info_sql_row.family_name,
      middle_name: user_info_sql_row.middle_name,
      nickname: user_info_sql_row.nickname,
      preferred_username: None,
      profile_picture: user_info_sql_row.profile_picture,
      website: user_info_sql_row.website,
      gender: user_info_sql_row.gender,
      birthdate: user_info_sql_row
        .birthdate
        .and_then(|birthdate| DateTime::<Utc>::from_timestamp(birthdate, 0)),
      zone_info: user_info_sql_row.zone_info,
      locale: user_info_sql_row.locale,
      address: user_info_sql_row.address,
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
  fn aud(&self) -> &str {
    &self.basic_claims.aud
  }
  fn sub(&self) -> i64 {
    self.basic_claims.sub
  }
  fn scope(&self) -> &str {
    &self.basic_claims.scope
  }
}
