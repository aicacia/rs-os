use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::core::helper::{json_to_string_vec, string_vec_to_json};

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
pub struct ClientAuthentication {
  pub client_id: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub client_secret: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub client_assertion: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub client_assertion_type: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(tag = "grant_type")]
pub enum TokenRequest {
  #[serde(rename = "password")]
  #[schema(title = "TokenRequestPassword")]
  Password {
    #[schema(example = "openid profile offline")]
    scope: String,
    #[schema(example = "admin")]
    username: String,
    #[schema(example = "admin")]
    password: String,
    #[serde(flatten)]
    #[schema(inline)]
    client_auth: ClientAuthentication,
  },
  #[serde(rename = "refresh_token")]
  #[schema(title = "TokenRequestRefreshToken")]
  RefreshToken {
    refresh_token: String,
    #[serde(flatten)]
    #[schema(inline)]
    client_auth: ClientAuthentication,
  },
  #[serde(rename = "authorization_code")]
  #[schema(title = "TokenRequestAuthorizationCode")]
  AuthorizationCode {
    code: String,
    code_verifier: Option<String>,
    #[serde(flatten)]
    #[schema(inline)]
    client_auth: ClientAuthentication,
  },
}

#[derive(Serialize, ToSchema)]
pub struct OpenIdConfiguration {
  pub issuer: String,
  pub authorization_endpoint: String,
  pub device_authorization_endpoint: String,
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

  pub fn needs_code(&self) -> bool {
    matches!(
      self,
      ResponseType::Code
        | ResponseType::CodeToken
        | ResponseType::CodeIdToken
        | ResponseType::CodeIdTokenToken
    )
  }

  pub fn needs_id_token(&self) -> bool {
    matches!(
      self,
      ResponseType::IdToken
        | ResponseType::CodeIdToken
        | ResponseType::IdTokenToken
        | ResponseType::CodeIdTokenToken
    )
  }

  pub fn needs_token(&self) -> bool {
    matches!(
      self,
      ResponseType::Token
        | ResponseType::CodeToken
        | ResponseType::IdTokenToken
        | ResponseType::CodeIdTokenToken
    )
  }
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct EndSessionRequest {
  pub client_id: Option<String>,
  pub id_token_hint: Option<String>,
  pub post_logout_redirect_uri: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RevokeRequest {
  pub token: String,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub token_type_hint: Option<String>,
  #[serde(flatten)]
  pub client_auth: ClientAuthentication,
}

#[derive(Deserialize, ToSchema, IntoParams)]
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
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub registration: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub code_challenge: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub code_challenge_method: Option<String>,
}

#[derive(Serialize, ToSchema, Default)]
pub struct Client {
  pub id: i64,
  pub active: bool,
  pub name: String,
  pub client_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_secret: Option<String>,
  pub redirect_uris: Option<Vec<String>>,
  pub post_logout_redirect_uris: Option<Vec<String>>,
  pub logo_uri: Option<String>,
  pub client_uri: Option<String>,
  pub policy_uri: Option<String>,
  pub terms_of_service_uri: Option<String>,
  pub application_type: String,
  pub auth_method: String,
  pub grant_types: Vec<String>,
  pub response_types: Vec<String>,
  pub scopes: Vec<String>,
  pub audience: Vec<String>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<os_model::entities::clients::Model> for Client {
  fn from(client_model: os_model::entities::clients::Model) -> Self {
    Self {
      active: client_model.is_active(),
      id: client_model.id,
      name: client_model.name,
      client_id: client_model.client_id,
      client_secret: Some(client_model.client_secret),
      redirect_uris: client_model.redirect_uris.map(json_to_string_vec),
      post_logout_redirect_uris: client_model
        .post_logout_redirect_uris
        .map(json_to_string_vec),
      logo_uri: client_model.logo_uri,
      client_uri: client_model.client_uri,
      policy_uri: client_model.policy_uri,
      terms_of_service_uri: client_model.terms_of_service_uri,
      application_type: client_model.application_type,
      auth_method: client_model.auth_method,
      grant_types: json_to_string_vec(client_model.grant_types),
      response_types: json_to_string_vec(client_model.response_types),
      scopes: json_to_string_vec(client_model.scopes),
      audience: json_to_string_vec(client_model.audience),
      access_token_expires_in_seconds: client_model.access_token_expires_in_seconds,
      id_token_expires_in_seconds: client_model.id_token_expires_in_seconds,
      refresh_expires_in_seconds: client_model.refresh_expires_in_seconds,
      updated_at: DateTime::<Utc>::from_timestamp(client_model.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(client_model.created_at, 0).unwrap_or_default(),
    }
  }
}

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
  pub audience: Vec<String>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
}

impl From<ClientRegisterRequest> for os_model::entities::clients::ActiveModel {
  fn from(request: ClientRegisterRequest) -> Self {
    use sea_orm::Set;

    let now = chrono::Utc::now().timestamp();
    Self {
      name: Set(request.name),
      client_id: Set(request.client_id),
      client_secret: Set(hex::encode(crate::core::encryption::random_bytes(64))),
      redirect_uris: Set(request.redirect_uris.as_ref().map(string_vec_to_json)),
      post_logout_redirect_uris: Set(
        request
          .post_logout_redirect_uris
          .as_ref()
          .map(string_vec_to_json),
      ),
      logo_uri: Set(request.logo_uri),
      client_uri: Set(request.client_uri),
      policy_uri: Set(request.policy_uri),
      terms_of_service_uri: Set(request.terms_of_service_uri),
      application_type: Set(request.application_type),
      auth_method: Set(request.auth_method),
      grant_types: Set(string_vec_to_json(&request.grant_types)),
      response_types: Set(string_vec_to_json(&request.response_types)),
      scopes: Set(string_vec_to_json(&request.scopes)),
      audience: Set(string_vec_to_json(&request.audience)),
      access_token_expires_in_seconds: Set(request.access_token_expires_in_seconds),
      id_token_expires_in_seconds: Set(request.id_token_expires_in_seconds),
      refresh_expires_in_seconds: Set(request.refresh_expires_in_seconds),
      active: Set(1),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    }
  }
}

#[derive(Deserialize, ToSchema)]
pub struct ClientAuthorizeRequest {
  pub client_id: String,
  pub scope: String,
  pub redirect_uri: String,
  pub response_type: ResponseType,
  pub code_challenge: Option<String>,
  pub code_challenge_method: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum ClientAuthorization {
  #[serde(rename = "authorization_code")]
  #[schema(title = "AuthorizationCode")]
  AuthorizationCode { code: String },
}

#[derive(Serialize, ToSchema, Default)]
pub struct ClientAllowed {
  pub allowed_scopes: Vec<String>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ClientByClientIdQuery {
  pub client_id: String,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ClientAllowedQuery {
  pub client_id: String,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ApproveClientQuery {
  pub client_id: String,
}
