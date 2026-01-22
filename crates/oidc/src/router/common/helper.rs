use chrono::{DateTime, Utc};
use http::StatusCode;
use os_api::{
  constants::{
    SCOPE_EMAIL, SCOPE_OFFLINE, SCOPE_PHONE, SCOPE_PROFILE, TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE,
    TOKEN_TYPE_BEARER, TOKEN_TYPE_ID, TOKEN_TYPE_REFRESH,
  },
  error::{HttpError, INTERNAL_ERROR},
};
use os_oidc_model::entities::{
  jwks::{get_jwk_for_sign_and_verify, model_to_jwt_jwk, to_encoding_key},
  user_emails, user_infos, user_phone_numbers, users,
};

use crate::{
  config::AppConfig,
  router::common::entity::{
    AuthorizationCodeClaims, BasicClaims, Claims, EncodeClaims, OpenIdClaims, OpenIdProfile, Token,
  },
};
use sea_orm::DatabaseConnection;

/// Parses a user URN (urn:os:sub:user:{id}) and extracts the user ID
pub fn parse_user_sub(sub: &str) -> Result<i64, String> {
  const PREFIX: &str = "urn:os:sub:user:";

  if let Some(id_str) = sub.strip_prefix(PREFIX) {
    id_str
      .parse::<i64>()
      .map_err(|e| format!("invalid user id in sub: {}", e))
  } else {
    Err(format!(
      "invalid sub format, expected '{}{{id}}', got '{}'",
      PREFIX, sub
    ))
  }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_user_token(
  db: &DatabaseConnection,
  app_config: &AppConfig,
  jwk_model: os_oidc_model::entities::jwks::Model,
  user: users::Model,
  client_id: String,
  audience: Vec<String>,
  scope: String,
  issued_token_type: String,
) -> Result<Token, HttpError> {
  let now = chrono::Utc::now();

  let issuer = app_config.url();
  let claims = BasicClaims {
    r#type: TOKEN_TYPE_BEARER.to_owned(),
    sub: format!("urn:os:sub:user:{}", user.id),
    aud: audience,
    user: user.id,
    client: client_id,
    iat: now.timestamp(),
    nbf: now.timestamp(),
    exp: now
      .timestamp()
      .saturating_add(app_config.token.expires_in_seconds as i64),
    iss: issuer.clone(),
    scope: scope.clone(),
  };

  let encoding_key = match to_encoding_key(&jwk_model) {
    Ok(encoding_key) => encoding_key,
    Err(e) => {
      log::error!("error getting converting into jwt encoding key: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };
  let jwk = match model_to_jwt_jwk(jwk_model.clone()) {
    Ok(jwk) => jwk,
    Err(e) => {
      log::error!("error getting converting into json web key: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };

  let access_token = match claims.encode(&issuer, &jwk, &encoding_key) {
    Ok(token) => token,
    Err(e) => {
      log::error!("error encoding access token: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };

  let (refresh_token, refresh_token_expires_in) = if claims.has_scope(SCOPE_OFFLINE) {
    let mut refresh_claims = claims.clone();
    refresh_claims.r#type = TOKEN_TYPE_REFRESH.to_owned();
    refresh_claims.exp = refresh_claims.iat + app_config.token.refresh_expires_in_seconds as i64;
    let refresh_token = match refresh_claims.encode(&issuer, &jwk, &encoding_key) {
      Ok(token) => token,
      Err(e) => {
        log::error!("error encoding refresh token: {}", e);
        return Err(
          HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
        );
      }
    };
    (
      Some(refresh_token),
      Some(app_config.token.refresh_expires_in_seconds as i64),
    )
  } else {
    (None, None)
  };

  let show_profile = claims.has_scope(SCOPE_PROFILE);
  let show_email = claims.has_scope(SCOPE_EMAIL);
  let show_phone_number = claims.has_scope(SCOPE_PHONE);

  let mut id_token = None;
  if show_profile || show_email || show_phone_number {
    let mut id_claims = OpenIdClaims {
      basic_claims: claims.clone(),
      profile: OpenIdProfile {
        preferred_username: Some(user.username.clone()),
        ..Default::default()
      },
      username: user.username.clone(),
    };
    id_claims.basic_claims.r#type = TOKEN_TYPE_ID.to_owned();

    let permissions = match users::get_user_role_permissions_by_user_id(db, user.id).await {
      Ok(roles_permissions) => roles_permissions
        .into_values()
        .flat_map(|permissions| permissions.into_iter().map(|permission| permission.uri))
        .collect::<Vec<_>>(),
      Err(e) => {
        log::error!("error fetching user permissions for client: {}", e);
        return Err(
          HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
        );
      }
    };

    id_claims.basic_claims.scope =
      format!("{} {}", id_claims.basic_claims.scope, permissions.join(" "));

    let user_info = match user_infos::get_user_info_by_user_id(db, user.id).await {
      Ok(Some(user_info)) => user_info,
      Ok(None) => {
        // Create a default user info model
        let now = chrono::Utc::now().timestamp();
        os_oidc_model::entities::user_infos::Model {
          user_id: user.id,
          given_name: None,
          family_name: None,
          middle_name: None,
          nickname: None,
          profile_picture: None,
          website: None,
          gender: None,
          birthdate: None,
          zone_info: None,
          locale: None,
          address: None,
          updated_at: now,
          created_at: now,
        }
      }
      Err(e) => {
        log::error!("error fetching user info: {}", e);
        return Err(
          HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
        );
      }
    };

    id_claims.profile = user_info.into();
    id_claims.profile.preferred_username = Some(user.username.clone());

    if show_email {
      match user_emails::get_user_primary_email(db, user.id).await {
        Ok(Some(email)) => {
          id_claims.profile.email_verified = Some(email.is_verified());
          id_claims.profile.email = Some(email.email);
        }
        Ok(None) => {}
        Err(e) => {
          log::error!("error fetching user email: {}", e);
          return Err(
            HttpError::from(StatusCode::INTERNAL_SERVER_ERROR)
              .with_application_error(INTERNAL_ERROR),
          );
        }
      }
    }
    if show_phone_number {
      match user_phone_numbers::get_user_primary_phone_number(db, user.id).await {
        Ok(Some(phone_number)) => {
          id_claims.profile.phone_verified = Some(phone_number.is_verified());
          id_claims.profile.phone = Some(phone_number.phone_number);
        }
        Ok(None) => {}
        Err(e) => {
          log::error!("error fetching user phone number: {}", e);
          return Err(
            HttpError::from(StatusCode::INTERNAL_SERVER_ERROR)
              .with_application_error(INTERNAL_ERROR),
          );
        }
      }
    }

    id_token = match id_claims.encode(&issuer, &jwk, &encoding_key) {
      Ok(token) => Some(token),
      Err(e) => {
        log::error!("error encoding id_token: {}", e);
        return Err(
          HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
        );
      }
    };
  }

  // Check if password reset is required
  let password_reset_required = match users::get_user_active_password_by_user_id(db, user.id).await
  {
    Ok(Some(user_password)) => {
      let is_expired = app_config
        .password
        .force_reset_after_days
        .and_then(|days| {
          if days == 0 {
            None
          } else {
            Some((days as i64) * 86400)
          }
        })
        .map(|max_age_seconds| user_password.is_password_expired(max_age_seconds))
        .unwrap_or(false);
      let is_reset_required = user_password.is_reset_required();

      if is_expired || is_reset_required {
        Some(true)
      } else {
        None
      }
    }
    Ok(None) => None,
    Err(e) => {
      log::error!("error checking password reset requirement: {}", e);
      None
    }
  };

  Ok(Token {
    access_token,
    token_type: claims.r#type,
    issued_token_type,
    issued_at: DateTime::<Utc>::from_timestamp(claims.iat, 0).unwrap_or_default(),
    expires_in: app_config.token.expires_in_seconds as i64,
    scope,
    refresh_token,
    refresh_token_expires_in,
    id_token,
    password_reset_required,
  })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn create_user_authorization_code_token(
  db: &DatabaseConnection,
  app_config: &AppConfig,
  user_id: i64,
  client_id: String,
  audience: Vec<String>,
  scope: String,
  code_challenge: String,
  code_challenge_method: String,
) -> Result<String, HttpError> {
  let jwk_model = match get_jwk_for_sign_and_verify(db).await {
    Ok(Some(jwk_model)) => jwk_model,
    Ok(None) => {
      log::error!("error no valid jwk for signing and verifying jwts");
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
    Err(e) => {
      log::error!("error getting jwk: {}", e);
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
  };

  let now = chrono::Utc::now();

  let issuer = app_config.url();
  let claims = AuthorizationCodeClaims {
    basic_claims: BasicClaims {
      r#type: TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE.to_owned(),
      sub: format!("urn:os:sub:user:{}", user_id),
      aud: audience,
      user: user_id,
      client: client_id.clone(),
      iat: now.timestamp(),
      nbf: now.timestamp(),
      exp: now
        .timestamp()
        .saturating_add(app_config.token.expires_in_seconds as i64),
      iss: issuer.clone(),
      scope,
    },
    code_challenge,
    code_challenge_method,
  };

  let encoding_key = match to_encoding_key(&jwk_model) {
    Ok(encoding_key) => encoding_key,
    Err(e) => {
      log::error!("error getting converting into jwt encoding key: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };
  let jwk = match model_to_jwt_jwk(jwk_model.clone()) {
    Ok(jwk) => jwk,
    Err(e) => {
      log::error!("error getting converting into json web key: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };

  let authorization_code_token = match claims.encode(&issuer, &jwk, &encoding_key) {
    Ok(token) => token,
    Err(e) => {
      log::error!("error encoding access token: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };

  Ok(authorization_code_token)
}

pub fn parse_jwt<T>(
  jwt: &str,
  app_config: &AppConfig,
  decoding_key: jsonwebtoken::DecodingKey,
  algorithm: jsonwebtoken::Algorithm,
) -> Result<jsonwebtoken::TokenData<T>, jsonwebtoken::errors::Error>
where
  T: Claims,
{
  let mut validation = jsonwebtoken::Validation::new(algorithm);
  validation.validate_nbf = true;
  validation.validate_aud = false;
  validation.set_issuer(&[app_config.url()]);

  jsonwebtoken::decode(jwt, &decoding_key, &validation)
}

pub fn to_public_jwk(jwk: &jsonwebtoken::jwk::Jwk) -> jsonwebtoken::jwk::Jwk {
  let mut public_jwk = jwk.clone();
  public_jwk.common.key_operations = public_jwk.common.key_operations.map(|key_operations| {
    key_operations
      .into_iter()
      .filter(is_public_key_operation)
      .collect()
  });
  public_jwk
}

pub fn is_public_key_operation(key_operation: &jsonwebtoken::jwk::KeyOperations) -> bool {
  matches!(
    key_operation,
    jsonwebtoken::jwk::KeyOperations::Verify
      | jsonwebtoken::jwk::KeyOperations::Encrypt
      | jsonwebtoken::jwk::KeyOperations::WrapKey
  )
}
