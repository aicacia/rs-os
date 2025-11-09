use std::collections::HashSet;

use axum::{
  extract::{Form, State},
  response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::{
    encryption::verify_password,
    jwk::sql::{get_jwk_for_sign_and_verify, list_jwks},
  },
  model::user::sql::{
    get_user_active_password_by_user_id, get_user_by_id, get_user_by_username_or_primary_email,
  },
  router::{
    common::{
      constants::{TOKEN_ISSUE_TYPE_PASSWORD, TOKEN_ISSUE_TYPE_REFRESH_TOKEN, TOKEN_TYPE_REFRESH},
      entity::{BasicClaims, Token},
      helper::create_user_token,
    },
    entity::RouterState,
    error::{CREDENTIALS, HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_ALLOWED_ERROR},
    middleware::authorization::parse_authorization,
    oidc::{
      constants::TAG,
      entity::{JWK, JWKs, OpenIdConfiguration, TokenRequest, TokenRequestCommon},
    },
  },
};

#[utoipa::path(
  get,
  path = "/.well-known/jwks.json",
  tags = [TAG],
  responses(
    (status = 200, description = "JSON Web Keys", body = JWKs),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn jwks(State(state): State<RouterState>) -> impl IntoResponse {
  let jwks = match list_jwks(&state.pool).await {
    Ok(jwks) => jwks,
    Err(e) => {
      log::error!("error getting jwks: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let keys = jwks
    .into_iter()
    .map(|jwk| JWK {
      key_ops: jwk.public_key_operations(),

      kid: jwk.kid.to_string(),
      kty: jwk.kty,
      alg: jwk.alg,
      r#use: jwk.r#use,

      n: jwk.n,
      e: jwk.e,

      crv: jwk.crv,
      x: jwk.x,
      y: jwk.y,

      x5c: jwk.x5c,
      x5u: jwk.x5u,
      x5t: jwk.x5t,
      x5t_s256: jwk.x5t_s256,
    })
    .collect();

  axum::Json(JWKs { keys }).into_response()
}

#[utoipa::path(
  get,
  path = "/.well-known/openid-configuration",
  tags = [TAG],
  responses(
    (status = 200, description = "OpenId Configuration", body = OpenIdConfiguration),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn openid_configuration(State(state): State<RouterState>) -> impl IntoResponse {
  let issuer = state.config.public_url();

  let jwks = match list_jwks(&state.pool).await {
    Ok(jwks) => jwks,
    Err(e) => {
      log::error!("error getting jwks: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let mut signing_algs = HashSet::new();

  for jwk in jwks {
    let key_operations = jwk.key_operations();

    log::info!("{:?}", key_operations);

    if key_operations
      .iter()
      .find(|s| s.as_str() == "sign")
      .is_some()
    {
      signing_algs.insert(jwk.alg.to_owned());
    }
  }

  axum::Json(OpenIdConfiguration {
    authorization_endpoint: format!("{}/authorize", issuer),
    device_authorization_endpoint: None, // Some(format!("{}/device-authorize", issuer)),
    token_endpoint: format!("{}/token", issuer),
    userinfo_endpoint: Some(format!("{}/current-user", issuer)),
    revocation_endpoint: Some(format!("{}/revoke", issuer)),
    jwks_uri: format!("{}/.well-known/jwks.json", issuer),
    response_types_supported: vec![
      "code".to_owned(),
      "token".to_owned(),
      "id_token".to_owned(),
      "code token".to_owned(),
      "code id_token".to_owned(),
      "token id_token".to_owned(),
      "code token id_token".to_owned(),
      "none".to_owned(),
    ],
    response_modes_supported: vec![
      "query".to_owned(),
      "fragment".to_owned(),
      "form_post".to_owned(),
    ],
    subject_types_supported: vec!["public".to_owned(), "pairwise".to_owned()],
    id_token_signing_alg_values_supported: signing_algs.into_iter().collect(),
    scopes_supported: vec![
      "openid".to_owned(),
      "profile".to_owned(),
      "address".to_owned(),
      "offline".to_owned(),
      "email".to_owned(),
      "phone_number".to_owned(),
    ],
    token_endpoint_auth_methods_supported: vec![
      "client_secret_post".to_owned(),
      "client_secret_basic".to_owned(),
      "none".to_owned(),
    ],
    claims_supported: vec![
      "sub".to_owned(),
      "aud".to_owned(),
      "exp".to_owned(),
      "iat".to_owned(),
      "iss".to_owned(),
      "name".to_owned(),
      "family_name".to_owned(),
      "given_name".to_owned(),
      "email".to_owned(),
      "email_verified".to_owned(),
      "phone_number".to_owned(),
      "phone_number_verified".to_owned(),
    ],
    code_challenge_methods_supported: vec!["plain".to_owned(), "S256".to_owned()],
    grant_types_supported: vec![
      "password".to_owned(),
      "authorization_code".to_owned(),
      "refresh_token".to_owned(),
    ],
    issuer,
  })
  .into_response()
}

#[utoipa::path(
  post,
  path = "/token",
  tags = [TAG],
  request_body(content = TokenRequest, content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = 201, description = "Token returned", body = Token),
    (status = 401, description = "Unauthorized Error", body = HttpError),
    (status = 403, description = "Forbiddon Error", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn token(
  State(state): State<RouterState>,
  Form(token_request): Form<TokenRequest>,
) -> impl IntoResponse {
  let token_result = match token_request {
    TokenRequest::Password {
      common,
      username,
      password,
    } => password_grant(state, common, username, password).await,
    TokenRequest::RefreshToken {
      common,
      refresh_token,
    } => refresh_token_grant(state, common, refresh_token).await,
    TokenRequest::AuthorizationCode { .. } => {
      Err(HttpError::internal_error().with_application_error(NOT_ALLOWED_ERROR))
    }
  };
  match token_result {
    Ok(token) => axum::Json(token).into_response(),
    Err(e) => e.into_response(),
  }
}

async fn password_grant(
  state: RouterState,
  common: TokenRequestCommon,
  username: String,
  password: String,
) -> Result<Token, HttpError> {
  let user = match get_user_by_username_or_primary_email(&state.pool, &username).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized().with_error(CREDENTIALS, INVALID_ERROR)),
    Err(e) => {
      log::error!("error getting user: {}", e);
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
  };

  let user_password = match get_user_active_password_by_user_id(&state.pool, user.id).await {
    Ok(Some(user_password)) => user_password,
    Ok(None) => {
      return Err(HttpError::forbidden().with_error(CREDENTIALS, TOKEN_ISSUE_TYPE_PASSWORD));
    }
    Err(e) => {
      log::error!("error fetching user password from database: {}", e);
      return Err(HttpError::unauthorized().with_application_error(INTERNAL_ERROR));
    }
  };

  match verify_password(&password, &user_password.encrypted_password) {
    Ok(true) => {}
    Ok(false) => {
      return Err(HttpError::unauthorized().with_error(CREDENTIALS, INVALID_ERROR));
    }
    Err(e) => {
      log::error!("error verifying user password: {}", e);
      return Err(HttpError::unauthorized().with_application_error(INTERNAL_ERROR));
    }
  }

  let jwk = match get_jwk_for_sign_and_verify(&state.pool).await {
    Ok(Some(jwk)) => jwk,
    Ok(None) => {
      log::error!("error no valid jwk for signing and verifying jwts");
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
    Err(e) => {
      log::error!("error getting jwk: {}", e);
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
  };

  create_user_token(
    &state.pool,
    &state.config,
    jwk,
    user,
    common.scope.or_else(|| Some("openid".to_owned())),
    TOKEN_ISSUE_TYPE_PASSWORD.to_owned(),
  )
  .await
}

async fn refresh_token_grant(
  state: RouterState,
  _common: TokenRequestCommon,
  refresh_token: String,
) -> Result<Token, HttpError> {
  let (token_data, jwk_sql_row) =
    parse_authorization::<BasicClaims>(&state.pool, &state.config, &refresh_token).await?;

  if token_data.claims.r#type != TOKEN_TYPE_REFRESH {
    return Err(HttpError::unauthorized());
  }

  let user = match get_user_by_id(&state.pool, token_data.claims.sub).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized()),
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return Err(HttpError::internal_error());
    }
  };

  create_user_token(
    &state.pool,
    &state.config,
    jwk_sql_row,
    user,
    Some(token_data.claims.scopes.join(" ").to_owned()),
    TOKEN_ISSUE_TYPE_REFRESH_TOKEN.to_owned(),
  )
  .await
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(jwks))
    .routes(routes!(openid_configuration))
    .routes(routes!(token))
    .with_state(state)
}
