use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::client::sql::ClientSQLCommon;

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
  #[schema(example = "openid profile offline")]
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
    #[schema(example = "admin")]
    username: String,
    #[schema(example = "admin")]
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
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub authorization_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub device_authorization_endpoint: Option<String>,
  pub token_endpoint: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub end_session_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub userinfo_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub revocation_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub registration_endpoint: Option<String>,
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

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
  Query,      // Authorization Code (default)
  Fragment,   // Implicit
  FormPost,   // POST back with hidden form fields
  WebMessage, // Silent authentication
}

impl ResponseMode {
  pub fn as_str(&self) -> &str {
    match self {
      ResponseMode::Query => "query",
      ResponseMode::Fragment => "fragment",
      ResponseMode::FormPost => "form_post",
      ResponseMode::WebMessage => "web_message",
    }
  }
}

#[derive(Deserialize, ToSchema)]
pub enum ResponseType {
  #[serde(rename = "none")]
  None,
  #[serde(rename = "code")]
  Code,
  #[serde(rename = "token")]
  Token,
  #[serde(rename = "id_token")]
  IdToken,
  #[serde(rename = "code token")]
  CodeToken,
  #[serde(rename = "code id_token")]
  CodeIdToken,
  #[serde(rename = "id_token token")]
  IdTokenToken,
  #[serde(rename = "code id_token token")]
  CodeIdTokenToken,
}

impl ResponseType {
  pub fn as_str(&self) -> &str {
    match self {
      ResponseType::None => "none",
      ResponseType::Code => "code",
      ResponseType::Token => "token",
      ResponseType::IdToken => "id_token",
      ResponseType::CodeToken => "code token",
      ResponseType::CodeIdToken => "code id_token",
      ResponseType::IdTokenToken => "id_token token",
      ResponseType::CodeIdTokenToken => "code id_token token",
    }
  }
}

#[derive(Deserialize, ToSchema)]
pub struct EndSessionRequest {
  pub client_id: String,
  pub post_logout_redirect_uri: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AuthorizeRequest {
  pub client_id: String,
  pub response_type: ResponseType,
  pub response_mode: ResponseMode,
  pub scope: String,
  pub redirect_uri: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub state: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub nonce: Option<String>,
}

pub type Client = crate::router::client::entity::Client;

#[derive(Deserialize, ToSchema)]
pub struct ClientRegisterRequest {
  pub name: String,
  pub client_id: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub redirect_uris: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub post_logout_redirect_uris: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub logo_uri: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub client_uri: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub policy_uri: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub terms_of_service_uri: Option<String>,
  pub application_type: String,
  pub auth_method: String,
  pub grant_types: Vec<String>,
  pub response_types: Vec<String>,
  pub scopes: Vec<String>,
  pub audience: Option<Vec<String>>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
}

impl Into<ClientSQLCommon> for ClientRegisterRequest {
  fn into(self) -> ClientSQLCommon {
    ClientSQLCommon {
      name: self.name,
      client_id: self.client_id,
      redirect_uris: self
        .redirect_uris
        .map(|v| serde_json::to_string(&v).unwrap()),
      post_logout_redirect_uris: self
        .post_logout_redirect_uris
        .map(|v| serde_json::to_string(&v).unwrap()),
      logo_uri: self.logo_uri,
      client_uri: self.client_uri,
      policy_uri: self.policy_uri,
      terms_of_service_uri: self.terms_of_service_uri,
      application_type: self.application_type,
      auth_method: self.auth_method,
      grant_types: serde_json::to_string(&self.grant_types).unwrap(),
      response_types: serde_json::to_string(&self.response_types).unwrap(),
      scopes: serde_json::to_string(&self.scopes).unwrap(),
      audience: self.audience.map(|v| serde_json::to_string(&v).unwrap()),
      access_token_expires_in_seconds: self.access_token_expires_in_seconds,
      id_token_expires_in_seconds: self.id_token_expires_in_seconds,
      refresh_expires_in_seconds: self.refresh_expires_in_seconds,
    }
  }
}
