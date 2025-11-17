use chrono::{DateTime, Utc};
use http::StatusCode;

use crate::{
  core::{
    config::app_config::AppConfig,
    jwk::{
      helper::to_encoding_key,
      sql::{JwkSQLRow, get_jwk_for_sign_and_verify},
    },
  },
  model::user::sql::{
    UserSQLRow, get_user_info_by_user_id, get_user_primary_email, get_user_primary_phone_number,
  },
  router::{
    common::{
      constants::{
        SCOPE_ADDRESS, SCOPE_EMAIL, SCOPE_OFFLINE, SCOPE_OPENID, SCOPE_PHONE_NUMBER, SCOPE_PROFILE,
        TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE, TOKEN_TYPE_BEARER, TOKEN_TYPE_ID, TOKEN_TYPE_REFRESH,
      },
      entity::{BasicClaims, Claims, OpenIdClaims, OpenIdProfile, Token},
    },
    error::{HttpError, INTERNAL_ERROR},
  },
};

pub(crate) async fn create_user_token(
  pool: &sqlx::AnyPool,
  app_config: &AppConfig,
  jwk_sql_row: JwkSQLRow,
  user: UserSQLRow,
  scope: Option<String>,
  issued_token_type: String,
  audiences: &[String],
) -> Result<Token, HttpError> {
  let now = chrono::Utc::now();
  let scopes = parse_scopes(scope.as_ref().map(String::as_str));

  let issuer = app_config.api_url();
  let claims = BasicClaims {
    r#type: TOKEN_TYPE_BEARER.to_owned(),
    sub: user.id,
    iat: now.timestamp(),
    nbf: now.timestamp(),
    exp: now.timestamp() + app_config.token.expires_in_seconds as i64,
    iss: issuer.clone(),
    aud: audiences.to_vec(),
    scopes: scopes.clone(),
  };

  let encoding_key = match to_encoding_key(&jwk_sql_row) {
    Ok(encoding_key) => encoding_key,
    Err(e) => {
      log::error!("error getting converting into jwt encoding key: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };
  let jwk = match jwk_sql_row.try_into() {
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

  let (refresh_token, refresh_token_expires_in) = if has_offline_scope(&scopes) {
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

  let show_profile = has_profile_scope(&scopes);
  let show_email = has_email_scope(&scopes);
  let show_phone_number = has_phone_number_scope(&scopes);

  let mut id_token = None;
  if show_profile || show_email || show_phone_number {
    let mut id_claims = OpenIdClaims {
      claims: claims.clone(),
      profile: OpenIdProfile {
        preferred_username: Some(user.username.clone()),
        ..Default::default()
      },
      ..Default::default()
    };
    id_claims.claims.r#type = TOKEN_TYPE_ID.to_owned();

    let user_info = match get_user_info_by_user_id(pool, user.id).await {
      Ok(Some(user_info)) => user_info,
      Ok(None) => Default::default(),
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
      match get_user_primary_email(pool, user.id).await {
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
      match get_user_primary_phone_number(pool, user.id).await {
        Ok(Some(phone_number)) => {
          id_claims.profile.phone_number_verified = Some(phone_number.is_verified());
          id_claims.profile.phone_number = Some(phone_number.phone_number);
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

  Ok(Token {
    access_token,
    token_type: claims.r#type,
    issued_token_type,
    issued_at: DateTime::<Utc>::from_timestamp(claims.iat, 0).unwrap_or_default(),
    expires_in: app_config.token.expires_in_seconds as i64,
    scope,
    refresh_token: refresh_token,
    refresh_token_expires_in: refresh_token_expires_in,
    id_token: id_token,
  })
}

pub(crate) async fn create_user_auhorization_code_token(
  pool: &sqlx::AnyPool,
  app_config: &AppConfig,
  user: UserSQLRow,
  audiences: &[String],
) -> Result<String, HttpError> {
  let jwk_sql_row = match get_jwk_for_sign_and_verify(pool).await {
    Ok(Some(jwk_sql_row)) => jwk_sql_row,
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

  let issuer = app_config.api_url();
  let claims = BasicClaims {
    r#type: TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE.to_owned(),
    sub: user.id,
    iat: now.timestamp(),
    nbf: now.timestamp(),
    exp: now.timestamp() + app_config.token.expires_in_seconds as i64,
    iss: issuer.clone(),
    aud: audiences.to_vec(),
    scopes: Vec::default(),
  };

  let encoding_key = match to_encoding_key(&jwk_sql_row) {
    Ok(encoding_key) => encoding_key,
    Err(e) => {
      log::error!("error getting converting into jwt encoding key: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };
  let jwk = match jwk_sql_row.try_into() {
    Ok(jwk) => jwk,
    Err(e) => {
      log::error!("error getting converting into json web key: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };

  let auhorization_code_token = match claims.encode(&issuer, &jwk, &encoding_key) {
    Ok(token) => token,
    Err(e) => {
      log::error!("error encoding access token: {}", e);
      return Err(
        HttpError::from(StatusCode::INTERNAL_SERVER_ERROR).with_application_error(INTERNAL_ERROR),
      );
    }
  };

  Ok(auhorization_code_token)
}

pub fn parse_scopes(scopes: Option<&str>) -> Vec<String> {
  match scopes {
    Some(scopes) => scopes.split(' ').map(|s| s.trim().to_owned()).collect(),
    None => vec![],
  }
}

pub fn has_profile_scope(scopes: &[String]) -> bool {
  scopes.contains(&SCOPE_PROFILE.to_owned())
}

pub fn has_openid_scope(scopes: &[String]) -> bool {
  scopes.contains(&SCOPE_OPENID.to_owned())
}

pub fn has_address_scope(scopes: &[String]) -> bool {
  scopes.contains(&SCOPE_ADDRESS.to_owned())
}

pub fn has_email_scope(scopes: &[String]) -> bool {
  scopes.contains(&SCOPE_EMAIL.to_owned())
}

pub fn has_phone_number_scope(scopes: &[String]) -> bool {
  scopes.contains(&SCOPE_PHONE_NUMBER.to_owned())
}

pub fn has_offline_scope(scopes: &[String]) -> bool {
  scopes.contains(&SCOPE_OFFLINE.to_owned())
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
  validation.set_issuer(&[app_config.api_url()]);

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
  match key_operation {
    jsonwebtoken::jwk::KeyOperations::Verify => true,
    jsonwebtoken::jwk::KeyOperations::Encrypt => true,
    jsonwebtoken::jwk::KeyOperations::WrapKey => true,
    _ => false,
  }
}
