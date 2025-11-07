use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Default, Serialize, ToSchema)]
pub struct JWK {
  pub kid: String,
  pub kty: String,
  pub alg: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub r#use: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub key_ops: Option<Vec<String>>,

  // RSA public fields
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub n: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub e: Option<String>,

  // EC public fields
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub crv: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub x: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub y: Option<String>,

  // X.509 fields
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub x5u: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub x5c: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub x5t: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub x5t_s256: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct JWKs {
  pub keys: Vec<JWK>,
}

#[derive(Deserialize, ToSchema)]
pub struct TokenRequestCommon {
  #[schema(example = "fda33145-9596-4294-904d-bb554202ce81")]
  pub client_id: Option<String>,
  #[schema(example = "openid")]
  pub scope: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(tag = "grant_type")]
pub enum TokenRequest {
  #[serde(rename = "password")]
  #[schema(title = "TokenRequestPassword")]
  Password {
    #[serde(flatten)]
    common: TokenRequestCommon,
    username: String,
    password: String,
  },
  #[serde(rename = "refresh_token")]
  #[schema(title = "TokenRequestRefreshToken")]
  RefreshToken {
    #[serde(flatten)]
    common: TokenRequestCommon,
    refresh_token: String,
  },
  #[serde(rename = "authorization_code")]
  #[schema(title = "TokenRequestAuthorizationCode")]
  AuthorizationCode {
    #[serde(flatten)]
    common: TokenRequestCommon,
    code: String,
  },
}

#[derive(Serialize, ToSchema)]
pub struct OpenIdConfiguration {
  pub issuer: String,
  pub authorization_endpoint: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub device_authorization_endpoint: Option<String>,
  pub token_endpoint: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub userinfo_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub revocation_endpoint: Option<String>,
  pub jwks_uri: String,
  pub response_types_supported: Vec<String>,
  pub response_modes_supported: Vec<String>,
  pub subject_types_supported: Vec<String>,
  pub id_token_signing_alg_values_supported: Vec<String>,
  pub scopes_supported: Vec<String>,
  pub token_endpoint_auth_methods_supported: Vec<String>,
  pub claims_supported: Vec<String>,
  pub code_challenge_methods_supported: Vec<String>,
  pub grant_types_supported: Vec<String>,
}
